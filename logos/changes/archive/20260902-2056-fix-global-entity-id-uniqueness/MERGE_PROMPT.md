# 合并指令 — fix-global-entity-id-uniqueness

## 变更提案
- 提案名称：fix-global-entity-id-uniqueness
- 提案目录：logos/changes/archive/fix-global-entity-id-uniqueness/

## 提案内容

修复"新账户建表保存 500"与"创建字段/关系时保存失败"——前端实体 id 全局唯一性 bug。

### 症状链

1. 用户反馈"创建字段关系时保存失败"
2. 后端日志：`UNIQUE constraint failed: reference.id`
3. commit `d0a63e7`（本会话初版修复）：`next_id` 从 store 现有 ids 解析 max+1
4. commit `a0fd920`（根因修复，李广桥）：用 `new_entity_id` splitmix64 + 原子计数器生成 `{prefix}-{16位hex}` 随机 id，根治

### 根因

前端 `auto-N` / `ref-N` 计数器 id：
- **已加载 diagram**：commit `d0a63e7` 修复——next_id 从 max+1 起步
- **新 diagram**：从空 store 开始计数（auto-1, ref-0），与后端**全局**主键（不限于当前 diagram）冲突 → UNIQUE 失败

后端 PK 是全局唯一（跨所有 diagram），前端计数器只关心当前 diagram。

### 修复

`a0fd920`：`new_entity_id(&store, "auto")` 返回 `{prefix}-{16位hex}`，splitmix64 + 原子计数器保证：
- 会话内确定唯一（原子计数器自增）
- 跨会话/跨机器唯一（时间戳 + 随机种子混入 splitmix64）
- wasm/host 双路径都适用

替换建表 / 新增字段 / 关系 / 区域 / 便签 6 处 auto-N/ref-N 计数器调用。

### 合并的提交

```
95a3179 docs(change): fix-global-entity-id-uniqueness 提案与 merge 标记
a0fd920 fix(editor): 实体 id 改全局唯一随机生成，修复新账户建表保存 500
```

附带修复（已在 work/experimental）：

```
d0a63e7 fix(id): next_id 从 store 现有 ids 解析 max+1，避免 UNIQUE 冲突
```

## 验收（用户终端跑）

```bash
cd /home/kyle/coldrawdb
openlogos verify && openlogos smoke
```

预期：
- `openlogos verify`：Gate 3.6 PASS（244+ pass / 0 fail / Coverage 100%）
- `openlogos smoke`：Gate 3.8 PASS（6/6）
- `cargo check / build` 与 `trunk build --release`：✅ pass
- 新增回归 UT-ID-GLOBAL-01 / UT-ID-GLOBAL-02（4000 id 唯一性 + 兼容存量 max+1 解析）

## 行为验收

| 场景 | 验证路径 |
|---|---|
| 新账户 / 新 diagram 创建表 | id 不再 "auto-1"，而是 "auto-{hex}"，全局唯一 |
| 创建关系 | id "ref-{hex}"，不再撞 DB |
| 已加载 diagram 创建关系 | commit `d0a63e7` 保障 next_id 不撞已存 ids |
| 不同会话/不同用户同时创建 | splitmix64 时间戳 + 种子混合保障 |

## 不在本变更范围

- Monaco wasm 完整挂载
- 22 个剩余 skip（spec-defined / 视觉回归 / 杂项 e2e）