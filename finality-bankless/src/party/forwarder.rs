use crate::{
    party::{AuthoritySubtaskCommon, Task},
    Future,
};
use bankless_bft::SpawnHandle;
use futures::{channel::oneshot, future::select, pin_mut};
use log::debug;

/// Runs the forwarder within a single session.
pub fn task<F: Future<Output = ()> + Send + 'static>(
    subtask_common: AuthoritySubtaskCommon,
    forwarder: F,
) -> Task {
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = async move {
        debug!(target: "bankless-party", "Running the forwarder task for {:?}", session_id);
        pin_mut!(forwarder);
        select(forwarder, exit).await;
        debug!(target: "bankless-party", "Forwarder task stopped for {:?}", session_id);
    };

    let handle = spawn_handle.spawn_essential("bankless/consensus_session_forwarder", task);
    Task::new(handle, stop)
}
