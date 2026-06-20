# 变更提案：implement-v2-auth

> module: core | created: 2026-06-20

## 变更原因

S03 鉴权 API/DB 规格已完成（`auth.yaml` + `coldrawdb-v2-auth.sql`），需落地 V2 后端 5 个 `/api/v1/auth/*` 端点及对应测试。

## 变更类型

代码级

## 变更范围

- 影响的 API：`auth.yaml` 5 端点
- 影响的 DB：`user` + `auth_token`（migration `0003_v2_auth`）
- 影响的编排测试：`core-S03-user-auth.json`（后续实现后运行）
- 测试用例：`core-S03-test-cases.md`（本批新增）

## 部署影响

- 是否需要部署：是（V2 auth 为 staging 新能力）
- 是否涉及数据迁移：是（`0003_v2_auth.up.sql`）
- 是否需要 smoke：否（auth 纳入后续 E2E）

## 变更概述

1. 新增 `backend/src/auth/` 服务层 + `auth_v1.rs` HTTP 路由。
2. 新增 migration `0003_v2_auth`。
3. 集成 Argon2id + JWT + HttpOnly refresh cookie。
4. 8 个集成测试 + OpenLogos reporter（UT-S03-01~07、ST-S03-01）。

## 本批覆盖用例

- UT-S03-01 ~ UT-S03-07
- ST-S03-01
