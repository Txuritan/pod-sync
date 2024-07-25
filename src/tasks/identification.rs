use crate::tasks::TaskStatus;

pub async fn identification(status: TaskStatus) {
    while status.is_active() {}
}
