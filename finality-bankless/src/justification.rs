use crate::{
    crypto::{Signature, SignatureV1},
    finalization::BlockFinalizer,
    metrics::Checkpoint,
    network, Metrics, SessionId,
};
use bankless_bft::{PartialMultisignature, SignatureSet};
use bankless_primitives::BANKLESS_ENGINE_ID;
use codec::{Decode, DecodeAll, Encode};
use futures::{channel::mpsc, Stream, StreamExt};
use futures_timer::Delay;
use log::{debug, error, warn};
use sc_client_api::HeaderBackend;
use sp_api::{BlockId, BlockT, NumberFor};
use sp_runtime::traits::Header;
use std::marker::PhantomData;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;

/// A proof of block finality, currently in the form of a sufficiently long list of signatures.
#[derive(Clone, Encode, Decode, Debug, PartialEq)]
pub struct BanklessJustification {
    pub(crate) signature: SignatureSet<Signature>,
}

pub(crate) trait Verifier<B: BlockT> {
    fn verify(&self, justification: &BanklessJustification, hash: B::Hash) -> bool;
}

/// Bunch of methods for managing frequency of sending justification requests.
pub(crate) trait JustificationRequestDelay {
    /// Decides whether enough time has elapsed.
    fn can_request_now(&self) -> bool;
    /// Notice block finalization.
    fn on_block_finalized(&mut self);
    /// Notice request sending.
    fn on_request_sent(&mut self);
}

pub(crate) struct SessionInfo<B: BlockT, V: Verifier<B>> {
    pub(crate) current_session: SessionId,
    pub(crate) last_block_height: NumberFor<B>,
    pub(crate) verifier: Option<V>,
}

/// Returns `SessionInfo` for the session regarding block with no. `number`.
pub(crate) trait SessionInfoProvider<B: BlockT, V: Verifier<B>> {
    fn for_block_num(&self, number: NumberFor<B>) -> SessionInfo<B, V>;
}

impl<F, B, V> SessionInfoProvider<B, V> for F
where
    B: BlockT,
    V: Verifier<B>,
    F: Fn(NumberFor<B>) -> SessionInfo<B, V>,
{
    fn for_block_num(&self, number: NumberFor<B>) -> SessionInfo<B, V> {
        self(number)
    }
}

/// A notification for sending justifications over the network.
#[derive(Clone)]
pub struct JustificationNotification<Block: BlockT> {
    /// The justification itself.
    pub justification: BanklessJustification,
    /// The hash of the finalized block.
    pub hash: Block::Hash,
    /// The ID of the finalized block.
    pub number: NumberFor<Block>,
}

pub(crate) struct JustificationHandlerConfig<B: BlockT, D: JustificationRequestDelay> {
    pub(crate) justification_request_delay: D,
    pub(crate) metrics: Option<Metrics<<B::Header as Header>::Hash>>,
    /// How long should we wait when the session verifier is not yet available.
    pub(crate) verifier_timeout: Duration,
    /// How long should we wait for any notification.
    pub(crate) notification_timeout: Duration,
}

pub(crate) struct JustificationHandler<B, V, RB, C, D, SI, F>
where
    B: BlockT,
    V: Verifier<B>,
    RB: network::RequestBlocks<B> + 'static,
    C: HeaderBackend<B> + Send + Sync + 'static,
    D: JustificationRequestDelay,
    SI: SessionInfoProvider<B, V>,
    F: BlockFinalizer<B>,
{
    session_info_provider: SI,
    block_requester: RB,
    client: Arc<C>,
    finalizer: F,
    config: JustificationHandlerConfig<B, D>,
    phantom: PhantomData<V>,
}

impl<B, V, RB, C, D, SI, F> JustificationHandler<B, V, RB, C, D, SI, F>
where
    B: BlockT,
    V: Verifier<B>,
    RB: network::RequestBlocks<B> + 'static,
    C: HeaderBackend<B> + Send + Sync + 'static,
    D: JustificationRequestDelay,
    SI: SessionInfoProvider<B, V>,
    F: BlockFinalizer<B>,
{
    pub(crate) fn new(
        session_info_provider: SI,
        block_requester: RB,
        client: Arc<C>,
        finalizer: F,
        config: JustificationHandlerConfig<B, D>,
    ) -> Self {
        Self {
            session_info_provider,
            block_requester,
            client,
            finalizer,
            config,
            phantom: PhantomData,
        }
    }

    fn handle_justification_notification(
        &mut self,
        notification: JustificationNotification<B>,
        verifier: V,
        last_finalized: NumberFor<B>,
        stop_h: NumberFor<B>,
    ) {
        let JustificationNotification {
            justification,
            number,
            hash,
        } = notification;

        if number <= last_finalized || number > stop_h {
            debug!(target: "afa", "Not finalizing block {:?}. Last finalized {:?}, stop_h {:?}", number, last_finalized, stop_h);
            return;
        };

        if !(verifier.verify(&justification, hash)) {
            warn!(target: "afa", "Error when verifying justification for block {:?} {:?}", number, hash);
            return;
        };

        debug!(target: "afa", "Finalizing block {:?} {:?}", number, hash);
        let finalization_res = self.finalizer.finalize_block(
            hash,
            number,
            Some((BANKLESS_ENGINE_ID, justification.encode())),
        );
        match finalization_res {
            Ok(()) => {
                self.config.justification_request_delay.on_block_finalized();
                debug!(target: "afa", "Successfully finalized {:?}", number);
                if let Some(metrics) = &self.config.metrics {
                    metrics.report_block(hash, Instant::now(), Checkpoint::Finalized);
                }
            }
            Err(e) => {
                error!(target: "afa", "Fail in finalization of {:?} {:?} -- {:?}", number, hash, e);
            }
        }
    }

    fn request_justification(&mut self, num: NumberFor<B>) {
        if self.config.justification_request_delay.can_request_now() {
            debug!(target: "afa", "Trying to request block {:?}", num);

            if let Ok(Some(header)) = self.client.header(BlockId::Number(num)) {
                debug!(target: "afa", "We have block {:?} with hash {:?}. Requesting justification.", num, header.hash());
                self.config.justification_request_delay.on_request_sent();
                self.block_requester
                    .request_justification(&header.hash(), *header.number());
            } else {
                debug!(target: "afa", "Cancelling request, because we don't have block {:?}.", num);
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        authority_justification_rx: mpsc::UnboundedReceiver<JustificationNotification<B>>,
        import_justification_rx: mpsc::UnboundedReceiver<JustificationNotification<B>>,
    ) {
        let import_stream = wrap_channel_with_logging(import_justification_rx, "import");
        let authority_stream = wrap_channel_with_logging(authority_justification_rx, "aggregator");
        let mut notification_stream = futures::stream::select(import_stream, authority_stream);

        loop {
            let last_finalized_number = self.client.info().finalized_number;
            let SessionInfo {
                verifier,
                last_block_height: stop_h,
                current_session,
            } = self
                .session_info_provider
                .for_block_num(last_finalized_number + 1u32.into());
            if verifier.is_none() {
                debug!(target: "afa", "Verifier for session {:?} not yet available. Waiting {}ms and will try again ...", current_session, self.config.verifier_timeout.as_millis());
                Delay::new(self.config.verifier_timeout).await;
                continue;
            }
            let verifier = verifier.expect("We loop until this is some.");

            match timeout(self.config.notification_timeout, notification_stream.next()).await {
                Ok(Some(notification)) => {
                    self.handle_justification_notification(
                        notification,
                        verifier,
                        last_finalized_number,
                        stop_h,
                    );
                }
                Ok(None) => panic!("Justification stream ended."),
                Err(_) => {} //Timeout passed
            }

            self.request_justification(stop_h);
        }
    }
}

fn wrap_channel_with_logging<B: BlockT>(
    channel: mpsc::UnboundedReceiver<JustificationNotification<B>>,
    label: &'static str,
) -> impl Stream<Item = JustificationNotification<B>> {
    channel
        .inspect(move |_| {
            debug!(target: "afa", "Got justification ({})", label);
        })
        .chain(futures::stream::iter(std::iter::from_fn(move || {
            error!(target: "afa", "Justification ({}) stream ended.", label);
            None
        })))
}

/// Old format of justifications, needed for backwards compatibility.
#[derive(Clone, Encode, Decode, Debug, PartialEq)]
pub(crate) struct BanklessJustificationV1 {
    pub(crate) signature: SignatureSet<SignatureV1>,
}

impl From<BanklessJustificationV1> for BanklessJustification {
    fn from(just_v1: BanklessJustificationV1) -> BanklessJustification {
        let size = just_v1.signature.size();
        let just_drop_id: SignatureSet<Signature> = just_v1
            .signature
            .into_iter()
            .fold(SignatureSet::with_size(size), |sig_set, (id, sgn)| {
                sig_set.add_signature(&sgn.into(), id)
            });
        BanklessJustification {
            signature: just_drop_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JustificationDecoding {
    V1(BanklessJustificationV1),
    V2(BanklessJustification),
    Err,
}

pub(crate) fn backwards_compatible_decode(justification_raw: Vec<u8>) -> JustificationDecoding {
    if let Ok(justification) = BanklessJustification::decode_all(&justification_raw) {
        JustificationDecoding::V2(justification)
    } else if let Ok(justification) = BanklessJustificationV1::decode_all(&justification_raw) {
        JustificationDecoding::V1(justification)
    } else {
        JustificationDecoding::Err
    }
}
