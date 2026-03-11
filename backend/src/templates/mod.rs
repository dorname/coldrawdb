use actix_web::{delete, get, post, web};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

use crate::common::{CommonResponse, ResponseCode, ResponseMessage};
use crate::entity::prelude::Template;
use crate::entity::template::ActiveModel as TemplateActiveModel;
use crate::entity::vo::TemplateVo;
use crate::error::DrawDBError;
use crate::next_id;

pub fn templates_routes(config: &mut web::ServiceConfig) {
    config.service(query_all_templates);
    config.service(query_template);
    config.service(add_template);
    config.service(update_template);
    config.service(delete_template);
}

#[get("/queryAll")]
async fn query_all_templates(
    db: web::Data<DatabaseConnection>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let templates = Template::find().all(conn).await?;
    let vos: Vec<TemplateVo> = templates.iter().map(TemplateVo::from).collect();
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(vos).unwrap()),
    ))
}

#[get("/query/{id}")]
async fn query_template(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let id = id.into_inner();
    let template = Template::find_by_id(id).one(conn).await?;
    if let Some(model) = template {
        let vo = TemplateVo::from(&model);
        Ok(CommonResponse::new(
            ResponseCode::Success,
            ResponseMessage::Success,
            Some(serde_json::to_value(vo).unwrap()),
        ))
    } else {
        Ok(CommonResponse::new(
            ResponseCode::NotFound,
            ResponseMessage::NotFound,
            None,
        ))
    }
}

#[post("/add")]
async fn add_template(
    db: web::Data<DatabaseConnection>,
    template: web::Json<TemplateVo>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let id = next_id();
    let vo = template.into_inner();
    let model = vo.convert_to_model(id.clone());
    let active = TemplateActiveModel::from(model);
    let inserted = active.insert(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(inserted).unwrap()),
    ))
}

#[post("/update")]
async fn update_template(
    db: web::Data<DatabaseConnection>,
    template: web::Json<TemplateVo>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let vo = template.into_inner();
    let id = vo.id.clone();
    let model = vo.convert_to_model(id);
    let active = TemplateActiveModel::from(model);
    let updated = active.update(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(updated).unwrap()),
    ))
}

#[delete("/delete/{id}")]
async fn delete_template(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let id = id.into_inner();
    let res = Template::delete_by_id(id).exec(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(res.rows_affected).unwrap()),
    ))
}

