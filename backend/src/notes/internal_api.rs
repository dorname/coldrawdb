use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use crate::entity::vo::NoteVo;
use crate::{entity::note::{ActiveModel as NoteActiveModel}, error::DrawDBError};
use crate::entity::note::{Column, Entity as Note};
/// 新增注释
pub async fn add_note(
    tx: &DatabaseTransaction,
    note_vo: NoteVo
) -> Result<bool, DrawDBError> {
    let note_model = note_vo.convert_to_note();
    let note_active_model = NoteActiveModel::from(note_model);
    Note::insert(note_active_model).exec(tx).await?;
    Ok(true)
}

/// 更新注释
pub async fn update_note(
    tx: &DatabaseTransaction,
    note_vo: NoteVo
) -> Result<bool, DrawDBError> {
    let note_model = note_vo.convert_to_note();
    let note_active_model = NoteActiveModel::from(note_model);
    Note::update(note_active_model).filter(Column::Id.eq(note_vo.id)).exec(tx).await?;
    Ok(true)
}

/// 删除注释
pub async fn delete_note(
    tx: &DatabaseTransaction,
    note_vo: NoteVo
) -> Result<bool, DrawDBError> {
    Note::delete_many().filter(Column::Id.eq(note_vo.id)).exec(tx).await?;
    Ok(true)
}
