pub mod deletion;
pub mod identification;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DeletionId(pub i64);

impl From<i64> for DeletionId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct IdentificationId(pub i64);

impl From<i64> for IdentificationId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
