use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub struct DidRegistryWorkerService {}

impl DidRegistryWorkerService {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn sweep(&self, _db: &DatabaseConnection, _did_id: &Uuid) -> anyhow::Result<()> {
        info!("DidRegistryWorkerService sweep");
        Ok(())
    }
}
