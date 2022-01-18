use crate::{
    data_io::DataStore,
    network::{BanklessNetworkData, RequestBlocks},
    party::{AuthoritySubtaskCommon, Task},
};
use bankless_bft::SpawnHandle;
use futures::channel::oneshot;
use log::debug;
use sc_client_api::Backend;
use sp_runtime::traits::Block;

/// Runs the data store within a single session.
pub fn task<B, C, BE, RB>(
    subtask_common: AuthoritySubtaskCommon,
    mut data_store: DataStore<B, C, BE, RB, BanklessNetworkData<B>>,
) -> Task
where
    B: Block,
    C: crate::ClientForBankless<B, BE> + Send + Sync + 'static,
    C::Api: bankless_primitives::BanklessSessionApi<B>,
    BE: Backend<B> + 'static,
    RB: RequestBlocks<B> + 'static,
{
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = {
        async move {
            debug!(target: "bankless-party", "Running the data store task for {:?}", session_id);
            data_store.run(exit).await;
            debug!(target: "bankless-party", "Data store task stopped for {:?}", session_id);
        }
    };

    let handle = spawn_handle.spawn_essential("bankless/consensus_session_data_store", task);
    Task::new(handle, stop)
}
