pub use frappe_storage::{QueueTask, TaskState};
pub const QUEUES: &[&str] = &["critical","high","default","low"];
#[cfg(test)] mod t { #[test] fn qs(){ assert_eq!(super::QUEUES.len(),4); } }
