use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};

use crate::{entity::vo::ReferenceVo, error::DrawDBError};
use crate::entity::reference::{Column, Model as ReferenceModel};
use crate::entity::reference::ActiveModel as ReferenceActiveModel;
use crate::entity::reference::Entity as Reference;

/// 批量新增引用
pub async fn add_references(
    tx: &DatabaseTransaction,
    reference_vos: Vec<ReferenceVo>
) -> Result<bool, DrawDBError> {
    let references_models = reference_vos
    .into_iter()
    .map(|vo| vo.convert_to_reference())
    .collect::<Vec<ReferenceModel>>();
    let references_active_models = references_models
    .into_iter()
    .map(|model| ReferenceActiveModel::from(model))
    .collect::<Vec<ReferenceActiveModel>>();
    Reference::insert_many(references_active_models).exec(tx).await?;
    Ok(true)
}

/// 批量删除引用
pub async fn delete_references(
    tx: &DatabaseTransaction,
    reference_vos: Vec<ReferenceVo>
) -> Result<bool, DrawDBError> {
    let ids = reference_vos.iter().map(|vo| vo.id.clone()).collect::<Vec<String>>();
    Reference::delete_many().filter(Column::Id.is_in(ids)).exec(tx).await?;
    Ok(true)
}