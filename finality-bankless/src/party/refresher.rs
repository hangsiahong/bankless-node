use crate::{
    data_io::{refresh_best_chain, BanklessDataFor},
    party::{AuthoritySubtaskCommon, Task},
};
use bankless_bft::SpawnHandle;
use futures::channel::oneshot;
use log::debug;
use parking_lot::Mutex;
use sc_client_api::Backend;
use sp_api::NumberFor;
use sp_consensus::SelectChain;
use sp_runtime::traits::Block;
use std::sync::Arc;

/// Runs the latest block refresher within a single session.
pub fn task<B, BE, SC, C>(
    subtask_common: AuthoritySubtaskCommon,
    select_chain: SC,
    client: Arc<C>,
    proposed_block: Arc<Mutex<BanklessDataFor<B>>>,
    last_block: NumberFor<B>,
) -> Task
where
    B: Block,
    C: crate::ClientForBankless<B, BE> + Send + Sync + 'static,
    C::Api: bankless_primitives::BanklessSessionApi<B>,
    BE: Backend<B> + 'static,
    SC: SelectChain<B> + 'static,
{
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = async move {
        debug!(target: "bankless-party", "Running the chain refresh task for {:?}", session_id);
        refresh_best_chain(select_chain, client, proposed_block, last_block, exit).await;
        debug!(target: "bankless-party", "Chain refresh task stopped for {:?}", session_id);
    };

    let handle = spawn_handle.spawn_essential("bankless/consensus_session_refresher", task);
    Task::new(handle, stop)
}
