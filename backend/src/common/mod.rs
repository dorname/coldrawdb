use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
impl CommonResponse {
    pub fn new(
        code: ResponseCode,
        message: ResponseMessage,
        data: Option<serde_json::Value>,
    ) -> Self {

        Self {
            code: code.to_i32(),
            message: message.to_string(),
            data,
        }
    }
}
/// 定义返回消息
/// 成功
/// 失败
/// 未授权
/// 禁止访问
/// 未找到
/// 服务器错误
#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseMessage {
    Success,
    Failed,
    Unauthorized,
    Forbidden,
    NotFound,
    Default(String),
}
impl ResponseMessage {
    // 将ResponseMessage转换为String
    pub fn to_string(&self) -> String {
        match self {
            ResponseMessage::Success => "成功".to_string(),
            ResponseMessage::Failed => "失败".to_string(),
            ResponseMessage::Unauthorized => "未授权".to_string(),
            ResponseMessage::Forbidden => "禁止访问".to_string(),
            ResponseMessage::NotFound => "未找到".to_string(),
            ResponseMessage::Default(message) => message.clone(),
        }
    }
}

/// 定义响应code
/// 200: 成功
/// 400: 失败
/// 401: 未授权
/// 403: 禁止访问
/// 404: 未找到
/// 500: 服务器错误
#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseCode {
    Success,
    Failed,
    Unauthorized,
    Forbidden,
    NotFound,
}
impl ResponseCode {
    // 将ResponseCode转换为i32
    pub fn to_i32(&self) -> i32 {
        match self {
            ResponseCode::Success => 200,
            ResponseCode::Failed => 400,
            ResponseCode::Unauthorized => 401,
            ResponseCode::Forbidden => 403,
            ResponseCode::NotFound => 404,
        }
    }
}
impl Responder for CommonResponse {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        HttpResponse::Ok().json(self)
    }
    type Body = BoxBody;
}
