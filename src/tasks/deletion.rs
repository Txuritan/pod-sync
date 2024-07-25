use crate::tasks::TaskStatus;

pub async fn deletion(status: TaskStatus) {
    while status.is_active() {}
}
