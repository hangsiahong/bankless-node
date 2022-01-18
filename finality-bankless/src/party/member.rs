use crate::{
    crypto::KeyBox,
    data_io::{DataProvider, FinalizationHandler},
    network::BanklessNetwork,
    party::{AuthoritySubtaskCommon, Task},
};
use bankless_bft::{Config, SpawnHandle};
use futures::channel::oneshot;
use log::debug;
use sp_runtime::traits::Block;

/// Runs the member within a single session.
pub fn task<B: Block>(
    subtask_common: AuthoritySubtaskCommon,
    multikeychain: KeyBox,
    config: Config,
    network: BanklessNetwork<B>,
    data_provider: DataProvider<B>,
    finalization_handler: FinalizationHandler<B>,
) -> Task {
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = {
        let spawn_handle = spawn_handle.clone();
        async move {
            debug!(target: "bankless-party", "Running the member task for {:?}", session_id);
            bankless_bft::run_session(
                config,
                network,
                data_provider,
                finalization_handler,
                multikeychain,
                spawn_handle,
                exit,
            )
            .await;
            debug!(target: "bankless-party", "Member task stopped for {:?}", session_id);
        }
    };

    let handle = spawn_handle.spawn_essential("bankless/consensus_session_member", task);
    Task::new(handle, stop)
}
