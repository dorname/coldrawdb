# 解决方案 02：后端接口测试用例

## 1. 目标与范围

### 目标

- 为现有 Legacy API 编写并可持续维护的 **集成测试**（HTTP 层）。
- 可基于路由/契约 **生成测试骨架**，再人工补全 body 与断言。

### 范围

- 覆盖 `backend/src/main.rs` 中注册的五大 scope：`/diagrams`、`/tables`、`/todos`、`/references`、`/templates`，以及根路由。
- 每个接口至少 1 条「正常路径」用例；关键接口可补充边界（如 404、空 body、非法 id）。

---

## 2. 前置条件

### 当前状态

- 仅 `backend/src/diagrams/mod.rs` 内有 `#[cfg(test)] mod tests`，且依赖真实 DB 做查询，**非 HTTP 接口测试**。
- 无 `actix_web::test::TestRequest` / `init_service` 的用法。
- 当前 CI（`.github/workflows/build.yml`）只做前端 lint 与 build，**未运行后端或 `cargo test`**。

### 需测试的接口清单

与代码保持一致，便于文档与实施对齐：

| Scope | Method | Path | Request Body | 说明 |
|-------|--------|------|---------------|------|
| 根 | GET | `/` | - | index |
| 根 | GET | `/hello/{name}` | - | hello |
| diagrams | GET | `/diagrams/queryAll` | - | 查询所有图表 |
| diagrams | GET | `/diagrams/query/{id}` | - | 按 id 查询图表 |
| diagrams | GET | `/diagrams/latest` | - | 查询最新图表 |
| diagrams | POST | `/diagrams/add` | DiagramVo | 新增图表 |
| diagrams | POST | `/diagrams/update` | DiagramVo | 更新图表 |
| diagrams | DELETE | `/diagrams/delete/{id}` | - | 删除图表 |
| tables | GET | `/tables/queryTables/{diagram_id}` | - | 查询图表下表与字段（当前唯一对外 tables 接口） |
| todos | GET | `/todos/hello` | - | 示例 |
| todos | GET | `/todos/query/{diagram_id}/{order_field}` | - | 按 diagram 查 todo，order_field 为排序字段 |
| todos | POST | `/todos/add` | TaskAddVo | 新增 todo |
| todos | POST | `/todos/update` | TaskUpdateVo | 更新 todo |
| todos | DELETE | `/todos/delete/{id}` | - | 删除 todo |
| references | POST | `/references/add` | Vec\<ReferenceVo\> | 新增引用 |
| references | POST | `/references/delete` | Vec\<ReferenceVo\> | 删除引用 |
| templates | GET | `/templates/queryAll` | - | 查询所有模板 |
| templates | GET | `/templates/query/{id}` | - | 按 id 查询模板 |
| templates | POST | `/templates/add` | TemplateVo | 新增模板 |
| templates | POST | `/templates/update` | TemplateVo | 更新模板 |
| templates | DELETE | `/templates/delete/{id}` | - | 删除模板 |

说明：`tables` 模块中有 `add` handler 但未在 `tables_routes` 中注册，故当前仅 `queryTables` 对外。

---

## 3. 技术选型

- **测试方式**：Actix-web 官方推荐——`actix_web::test::TestRequest` + `init_service(App::new().configure(...))`；或从 `main` 抽离 `app_config()`，在 main 与测试中共用。
- **数据库**：测试用 **内存 SQLite**（`sqlite::memory:`）或独立 `test.sqlite`；在测试 setup 中执行 `backend/migrations/` 的迁移或现有 `init` 逻辑，保证幂等、不依赖外部 config。

---

## 4. 分步实施

| 步骤 | 内容 |
|------|------|
| **Step 1** | 在 `backend` 下增加测试用 DB 初始化（例如 `init(true)` 或仅调用 `apply_migrations`），使用内存 DB 或固定路径的 test.sqlite，不读外部 config。 |
| **Step 2** | 将 App 配置抽离为函数（如 `fn app_config(cfg: &mut web::ServiceConfig)`），在 `main` 与测试中共用；测试中 `App::new().app_data(db).configure(app_config).bind("0.0.0.0:0")?` 或使用 `init_service`。 |
| **Step 3** | 按 scope 分文件或分 mod：如 `tests/api_diagrams.rs`、`api_tables.rs`、`api_todos.rs`、`api_references.rs`、`api_templates.rs`，每文件对应一个 scope；根路由可放在 `api_health.rs` 或合并到任一文件。 |
| **Step 4** | 为每个接口写至少 1 个测试：构造 `TestRequest`（GET/POST/DELETE + URI + 可选 body），`call_service(&mut app, req).await`，断言 `status().is_success()` 或具体 status，以及 body 中 `CommonResponse` 的 `code`/`data`（解析 JSON）。 |
| **Step 5** | （可选）「生成」测试骨架：写脚本（Rust 二进制或 Python）按接口清单生成各 `tests/api_*.rs` 中仅包含 `TestRequest` 与空断言的骨架，人工补 body 与断言。 |

---

## 5. 验收标准

- `cargo test` 在 `backend` 目录下全部通过。
- 文档中注明：若 CI 尚未包含后端测试，需在 `.github/workflows/` 中增加一步（例如单独 job：`cd backend && cargo test`）。

---

## 6. 参考与附录

### 附录 A：完整接口清单表（复制用）

| Method | Path | Request Body 类型 | 预期 200 响应要点 |
|--------|------|-------------------|--------------------|
| GET | `/` | - | 文本 "Hello, world!" |
| GET | `/hello/{name}` | - | 文本 "Hello, {name}!" |
| GET | `/diagrams/queryAll` | - | CommonResponse.code=200, data 为数组 |
| GET | `/diagrams/query/{id}` | - | code=200 有数据，或 404 无数据 |
| GET | `/diagrams/latest` | - | code=200 有数据，或 404 |
| POST | `/diagrams/add` | DiagramVo | code=200, data 含 id |
| POST | `/diagrams/update` | DiagramVo | code=200 |
| DELETE | `/diagrams/delete/{id}` | - | code=200 |
| GET | `/tables/queryTables/{diagram_id}` | - | code=200, data 为表列表 |
| GET | `/todos/hello` | - | 文本 |
| GET | `/todos/query/{diagram_id}/{order_field}` | - | code=200, data 为数组 |
| POST | `/todos/add` | TaskAddVo | code=200 |
| POST | `/todos/update` | TaskUpdateVo | code=200 |
| DELETE | `/todos/delete/{id}` | - | code=200 |
| POST | `/references/add` | Vec\<ReferenceVo\> | code=200 |
| POST | `/references/delete` | Vec\<ReferenceVo\> | code=200 |
| GET | `/templates/queryAll` | - | code=200, data 为数组 |
| GET | `/templates/query/{id}` | - | code=200 或 404 |
| POST | `/templates/add` | TemplateVo | code=200 |
| POST | `/templates/update` | TemplateVo | code=200 |
| DELETE | `/templates/delete/{id}` | - | code=200 |

### 附录 B：示例测试代码片段

**diagrams queryAll（GET，无 body）**

```rust
use actix_web::{test, web, App};
use backend::diagrams::diagrams_routes;

#[actix_web::test]
async fn test_diagrams_query_all() {
    let db = setup_test_db().await; // 需自己实现：内存 DB + 迁移
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .service(web::scope("/diagrams").configure(diagrams_routes)),
    )
    .await;

    let req = test::TestRequest::get().uri("/diagrams/queryAll").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert!(json["data"].is_array());
}
```

**diagrams add（POST，JSON body）**

```rust
#[actix_web::test]
async fn test_diagrams_add() {
    let db = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .service(web::scope("/diagrams").configure(diagrams_routes)),
    )
    .await;

    let payload = r#"{"id":"","title":"Test","database":"generic","tables":[],"relationships":[],"notes":[],"areas":[],"todos":[]}"#;
    let req = test::TestRequest::post()
        .uri("/diagrams/add")
        .set_payload(payload)
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert!(json["data"]["id"].as_str().unwrap().len() > 0);
}
```

实际实施时需将 `backend::diagrams::diagrams_routes` 改为项目内实际模块路径，并实现 `setup_test_db()`（例如 `Database::connect("sqlite::memory:")` + `apply_migrations`）。

### 相关代码位置

- 路由注册：`backend/src/main.rs`
- 各 scope 实现：`backend/src/diagrams/mod.rs`、`tables/mod.rs`、`todos/mod.rs`、`references/mod.rs`、`templates/mod.rs`
- 公共响应：`backend/src/common/mod.rs`（CommonResponse、ResponseCode）
- 迁移：`backend/migrations/`、`backend/src/init.rs`（apply_migrations）
