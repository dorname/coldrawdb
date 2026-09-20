# Delta — core-01-deployment-plan.md（trunk 配置文件名归一为小写）

> 变更：fix-trunk-toml-case-conflict

## MODIFIED — §4.1 镜像构建（wasm-build 阶段 COPY 的 trunk 配置文件）

原因：仓库索引中曾同时存在 `frontend-rs/Trunk.toml` 与 `frontend-rs/trunk.toml`（仅大小写不同、内容不同）。Windows 检出必然出现「假修改」，Linux 下 trunk 与 Dockerfile 又会优先命中只含 `[watch]` 的那一份，导致 `[tools] wasm-opt = false` / `[serve] no_autoreload` / `[build]` 丢失。现已归一为单一小写 `frontend-rs/trunk.toml`，本节构建片段同步改为小写。

将 wasm-build 阶段中的：

```dockerfile
COPY frontend-rs/index.html frontend-rs/Trunk.toml ./frontend-rs/
```

替换为：

```dockerfile
COPY frontend-rs/index.html frontend-rs/trunk.toml ./frontend-rs/
```

其余构建步骤不变。`wasm-opt` 仍由 `frontend-rs/trunk.toml` 的 `[tools] wasm-opt = false` 与 `frontend-rs/index.html` 的 `data-wasm-opt="0"` 关闭。
