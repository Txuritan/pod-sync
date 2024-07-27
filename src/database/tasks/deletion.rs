use uuid::Uuid;

use crate::{
    database::{tasks::DeletionId, user::User, Database},
    models::subscriptions::{Deletion, DeletionStatus},
};

impl Database {
    pub async fn deletion_create(&self, user: &User, uuid: Uuid) -> anyhow::Result<Option<i64>> {
        // TODO: create a version that is bound by the user
        let Some(row) = self.subscription_get_id_by_guid(uuid).await? else {
            return Ok(None);
        };

        let row = sqlx::query!(
            r#"--sql
                INSERT INTO task_deletions(user_id, subscription_id, status)
                VALUES (?, ?, ?)
                RETURNING id
            "#,
            user.id,
            row.subscription_id,
            DeletionStatus::Pending,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| row.id))
    }

    pub async fn deletion_get(
        &self,
        user: &User,
        id: DeletionId,
    ) -> anyhow::Result<Option<Deletion>> {
        let row = sqlx::query!(
            r#"--sql
                SELECT status as "status: DeletionStatus"
                FROM task_deletions
                WHERE id = ?1 AND user_id = ?2
            "#,
            id,
            user.id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| match row.status {
            DeletionStatus::Success => Deletion::success(id.0),
            DeletionStatus::Pending => Deletion::pending(id.0),
            DeletionStatus::Failure => Deletion::failure(id.0),
        }))
    }
}
