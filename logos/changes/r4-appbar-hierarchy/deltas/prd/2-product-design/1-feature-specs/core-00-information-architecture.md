## MODIFIED — §1 顶层布局（V2 — E1 z-index 扩展）

**merge 时在 AppBar 行描述后追加 R4 信息分层说明**：

### §1.1 AppBar 信息架构（R4）

| 分区 | 职责 | 用户心智 |
|---|---|---|
| 品牌区 | 项目身份 + 编辑历史 + 标题 | 「我在编辑哪个 diagram」 |
| 状态区 | 保存/sync 单一 Chip | 「数据是否安全」 |
| 操作区 | 协作（分享）+ 视图（代码）+ 低频 IO/主题/侧栏 | 「对外动作 vs 工具设置」 |

AppBar ASCII（R4）：

```
| Logo Undo Redo [Title________] | ● 已保存 · rev:5 |     [分享][<>][⋯] |
```

StatusBar 保留缩放/计数/Inspector 折叠；**不再**重复 `revision-display`。
