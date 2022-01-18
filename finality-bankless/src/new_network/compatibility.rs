// All this should be removed after the old network is no longer in use.
// In particular I use the above to avoid worrying about the code duplication below.
// It cannot easily be avoided, because it's kinda hard to make BanklessNetwork and RmcNetwork
// implement appropriate DataNetworks.
use crate::{
    network::{BanklessNetwork, RmcNetwork},
    new_network::{BanklessNetworkData, DataNetwork, RmcNetworkData, SendError},
};
use bankless_bft::{Network, Recipient};
use sp_runtime::traits::Block;

pub struct SplicedBanklessNetwork<B: Block, DN: DataNetwork<BanklessNetworkData<B>>> {
    new_bankless_network: DN,
    old_bankless_network: BanklessNetwork<B>,
}

impl<B: Block, DN: DataNetwork<BanklessNetworkData<B>>> SplicedBanklessNetwork<B, DN> {
    pub(crate) fn new(new_bankless_network: DN, old_bankless_network: BanklessNetwork<B>) -> Self {
        SplicedBanklessNetwork {
            new_bankless_network,
            old_bankless_network,
        }
    }
}

#[async_trait::async_trait]
impl<B: Block, DN: DataNetwork<BanklessNetworkData<B>>> DataNetwork<BanklessNetworkData<B>>
    for SplicedBanklessNetwork<B, DN>
{
    fn send(&self, data: BanklessNetworkData<B>, recipient: Recipient) -> Result<(), SendError> {
        let _ = self.old_bankless_network.send(data.clone(), recipient.clone());
        self.new_bankless_network.send(data, recipient)
    }

    async fn next(&mut self) -> Option<BanklessNetworkData<B>> {
        tokio::select! {
            data = self.old_bankless_network.next_event() => data,
            data = self.new_bankless_network.next() => data,
        }
    }
}

pub struct SplicedRmcNetwork<B: Block, DN: DataNetwork<RmcNetworkData<B>>> {
    new_rmc_network: DN,
    old_rmc_network: RmcNetwork<B>,
}

impl<B: Block, DN: DataNetwork<RmcNetworkData<B>>> SplicedRmcNetwork<B, DN> {
    pub(crate) fn new(new_rmc_network: DN, old_rmc_network: RmcNetwork<B>) -> Self {
        SplicedRmcNetwork {
            new_rmc_network,
            old_rmc_network,
        }
    }
}

#[async_trait::async_trait]
impl<B: Block, DN: DataNetwork<RmcNetworkData<B>>> DataNetwork<RmcNetworkData<B>>
    for SplicedRmcNetwork<B, DN>
{
    fn send(&self, data: RmcNetworkData<B>, recipient: Recipient) -> Result<(), SendError> {
        let _ = self
            .old_rmc_network
            .send(data.clone(), recipient.clone().into());
        self.new_rmc_network.send(data, recipient)
    }

    async fn next(&mut self) -> Option<RmcNetworkData<B>> {
        tokio::select! {
            data = self.old_rmc_network.next() => data,
            data = self.new_rmc_network.next() => data,
        }
    }
}
