use serde::{Deserialize, Serialize};    

#[derive(Serialize, Deserialize)]
struct TableVo {
    id: String,
    name: String,
    description: String,
    created_at: String,
    updated_at: String,
    fields: Vec<FieldVo>,
}


#[derive(Serialize, Deserialize)]
struct FieldVo {
    id: String,
    name: String,
    description: String,
}