# 合并指令 — fix-canvas-interaction-hotfixes

## 变更提案
- 提案名称：fix-canvas-interaction-hotfixes
- 提案目录：logos/changes/archive/20260827-0936-fix-canvas-interaction-hotfixes/

## 提案内容

3 个独立用户报告的交互 bug，根因各异，统一归档：

### 子问题 1：选中关系后画布无法拖动
- 根因：on_pointerdown rel_tool_active 未命中字段直接 return；
  on_pointerup 缺 endpoint_drag 显式处理；无 pointercancel 监听
- 修复 commit：a17b710

### 子问题 2：登录状态刷新丢失 + 表重叠在原点
- 根因 1：AuthSession 是纯内存 RwSignal，无 localStorage 持久化
- 根因 2：backend row_f64 用 try_get::<String> 取 INTEGER 列失败 → fallback 0.0
- 修复 commit：e0413fb

### 子问题 3：滚轮缩放内容漂出 + pan 方向错位
- 根因：on_wheel 反向计算 new_pan 缺少 rect.left 偏移
- 修复 commit：3b41cd4

### 附加修复（用户主动报告后追加）
- 66ffa9c：恢复 session 仅在非 share/invite 路径生效（避免覆盖 ST-S02-SHARE-VS-AUTH）
- 7ab27ab：verify 沙箱 truncate 容错（pre_run 不再因只读 workspace 立即 exit）
- db123e7：reporter 走 COLDRAWDB_JSONL_PATH 绝对路径，沙箱不丢结果

## 合并的提交

```
a17b710 fix(editor-render): 修复选中关系后画布无法拖动的 bug
e0413fb fix(auth+canvas): 登录持久化 + 表 x/y 读取修复
22cd454 docs(hotfixes): 新建 fix-canvas-interaction-hotfixes 归档提案
3b41cd4 fix(editor-render): 修复滚轮缩放 pan 累积漂移
7ab27ab fix(verify): 沙箱只读导致 jsonl 丢失 + truncate 容错
66ffa9c fix(auth): 恢复 session 仅在非 share/invite 路径生效
db123e7 fix(verify): reporter 走 COLDRAWDB_JSONL_PATH 绝对路径，沙箱不丢结果
```

## 验收

- `cargo check / cargo build --release` (backend + mcp-server)：✅ pass
- `trunk build --release` (frontend-rs)：✅ pass
- `openlogos verify`：Gate 3.6 PASS（244/266 pass, 0 fail, 22 skip, Coverage 100%）
- `openlogos smoke`：Gate 3.8 PASS（6/6 PASS, Coverage 100%）

## 行为验收

| 场景 | 验证路径 |
|---|---|
| 选中关系后画布可拖动 | a17b710：点击 endpoint 后空白处 pan，命中表可拖 |
| 登录刷新保持登录 + 表位置正确 | e0413fb：test@163.com / Test1234New 登录后 F5 |
| 滚轮缩放内容不漂 + pan 方向正确 | 3b41cd4：滚轮缩放 5 次后内容仍在 viewport |
| 分享链接登录后仍只读 | 66ffa9c：登录后访问 /?share=xxx 应只读 |

## 后续待办（不在本变更范围）

- Monaco wasm 完整挂载
- 22 个剩余 skip（spec-defined / 视觉回归 / 杂项 e2e）
- ST-FE-PROTO-* 像素相似度基线