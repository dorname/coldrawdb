# 变更提案：align-v1-api-completion

> module: core | created: 2026-06-21

## 变更原因

前后端对齐 **批次 B**：V1 bridge 5 端点中前端仅调用 `import/local`；`DiagramClient::delete` 无 UI；`bridge.yaml` 与后端 `phase3_bridge.rs` 契约不一致。

## 变更类型

接口级 + 代码级

## 变更范围

- API：`bridge.yaml` 对齐实际 `{ source, payload }` 与 `BridgeConfig` 字段
- 前端：`DiagramClient` 补全 bridge config / logs / retry；溢出菜单删除图；设置模态；ImportDrawer 日志区
- 测试：UT-ALIGN-B01~B03

## 部署影响

- 是否需要部署：否 | smoke：否 | 数据迁移：否

## 变更概述

1. 扩展 `DiagramClient`：GET/PUT bridge/config、GET import/logs、POST retry
2. AppBar 溢出菜单：设置（Bridge 配置）、删除当前图
3. ImportDrawer 底部：最近导入日志 + 失败重试
4. 更新 `bridge.yaml` 与实现对齐
