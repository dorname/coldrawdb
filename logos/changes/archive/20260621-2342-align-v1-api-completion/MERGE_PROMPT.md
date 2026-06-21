# 合并指令

## 变更提案
- 提案名称：align-v1-api-completion
- 提案目录：logos/changes/align-v1-api-completion/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/api/bridge.yaml

- Delta 文件：`logos/changes/align-v1-api-completion/deltas/api/bridge.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/align-v1-api-completion/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(align-v1-api-completion): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive align-v1-api-completion`。
