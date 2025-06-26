use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use crate::{entity::vo::IndiceVo, error::DrawDBError};
use crate::entity::indice::{ActiveModel as IndiceActiveModel, Column, Entity as Indice};
/// 新增索引
pub async fn add_indice(
    tx: &DatabaseTransaction,
    indice_vo: IndiceVo
) -> Result<bool, DrawDBError> {
    let indice_model = indice_vo.convert_to_indice();
    let indice_active_model = IndiceActiveModel::from(indice_model);
    Indice::insert(indice_active_model).exec(tx).await?;
    Ok(true)
}

/// 更新索引
pub async fn update_indice(
    tx: &DatabaseTransaction,
    indice_vo: IndiceVo
) -> Result<bool, DrawDBError> {
    let indice_model = indice_vo.convert_to_indice();
    let indice_active_model = IndiceActiveModel::from(indice_model);
    Indice::update(indice_active_model).filter(Column::Id.eq(indice_vo.id)).exec(tx).await?;
    Ok(true)
}

/// 删除索引
pub async fn delete_indice(
    tx: &DatabaseTransaction,
    indice_vo: IndiceVo
) -> Result<bool, DrawDBError> {
    Indice::delete_many().filter(Column::Id.eq(indice_vo.id)).exec(tx).await?;
    Ok(true)
}
