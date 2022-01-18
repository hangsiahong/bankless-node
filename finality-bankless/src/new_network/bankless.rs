use crate::{
    crypto::Signature,
    data_io::{BanklessDataFor, BanklessNetworkMessage},
    new_network::DataNetwork,
    Hasher,
};
use bankless_bft::{Network as BanklessNetwork, NetworkData as BanklessNetworkData, SignatureSet};
use log::warn;
use sp_runtime::traits::Block;
use std::marker::PhantomData;

pub type NetworkData<B> =
    BanklessNetworkData<Hasher, BanklessDataFor<B>, Signature, SignatureSet<Signature>>;

impl<B: Block> BanklessNetworkMessage<B> for NetworkData<B> {
    fn included_blocks(&self) -> Vec<BanklessDataFor<B>> {
        self.included_data()
    }
}

/// A wrapper needed only because of type system theoretical constraints. Sadness.
pub struct NetworkWrapper<B: Block, ADN: DataNetwork<NetworkData<B>>> {
    inner: ADN,
    phantom: PhantomData<B>,
}

impl<B: Block, ADN: DataNetwork<NetworkData<B>>> From<ADN> for NetworkWrapper<B, ADN> {
    fn from(inner: ADN) -> Self {
        NetworkWrapper {
            inner,
            phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B: Block, ADN: DataNetwork<NetworkData<B>>>
    BanklessNetwork<Hasher, BanklessDataFor<B>, Signature, SignatureSet<Signature>>
    for NetworkWrapper<B, ADN>
{
    fn send(&self, data: NetworkData<B>, recipient: bankless_bft::Recipient) {
        if self.inner.send(data, recipient).is_err() {
            warn!(target: "bankless-network", "Error sending an BanklessBFT message to the network.");
        }
    }

    async fn next_event(&mut self) -> Option<NetworkData<B>> {
        self.inner.next().await
    }
}
