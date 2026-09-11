use crate::entity::diagram::{ActiveModel, Model};
use crate::entity::prelude::Diagram;
use crate::error::DrawDBError;
use crate::next_id;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, TransactionTrait};

pub struct DiagramRepository;

impl DiagramRepository {
    pub async fn query_all(db: &DatabaseConnection) -> Result<Vec<Model>, DrawDBError> {
        Ok(Diagram::find().all(db).await?)
    }

    pub async fn query_by_id(db: &DatabaseConnection, id: &str) -> Result<Option<Model>, DrawDBError> {
        Ok(Diagram::find_by_id(id.to_string()).one(db).await?)
    }

    pub async fn create(db: &DatabaseConnection, mut model: Model) -> Result<Model, DrawDBError> {
        let tx = db.begin().await?;
        if model.id.is_empty() {
            model.id = next_id();
        }
        let saved = ActiveModel::from(model).insert(&tx).await?;
        tx.commit().await?;
        Ok(saved)
    }

    pub async fn update(db: &DatabaseConnection, model: ActiveModel) -> Result<Model, DrawDBError> {
        let tx = db.begin().await?;
        let saved = model.update(&tx).await?;
        tx.commit().await?;
        Ok(saved)
    }

    pub async fn delete(db: &DatabaseConnection, id: &str) -> Result<(), DrawDBError> {
        let tx = db.begin().await?;
        Diagram::delete_by_id(id.to_string()).exec(&tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
