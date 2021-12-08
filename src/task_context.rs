use crate::database::ID;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub exit_tx: ExitSender,
    pub connected_player: ID,
    pub task_perms: ID,
}

impl TaskContext {
    #[must_use]
    pub fn new(exit_tx: ExitSender, player: ID) -> Self {
        Self {
            exit_tx,
            connected_player: player,
            task_perms: player,
        }
    }

    pub fn shared(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }
}

pub type ExitSender = tokio::sync::mpsc::UnboundedSender<()>;
pub type SharedTaskContext = Arc<RwLock<TaskContext>>;

tokio::task_local! {
    pub static TASK_CONTEXT: SharedTaskContext;
}
