#!/usr/bin/env python3
"""Generate 5 substantive deltas (D-01 4 files + D-04 deployment).

Run from repo root: python3 logos/changes/add-baseline-docs/scripts/build-substantive-deltas.py
"""
from __future__ import annotations
from pathlib import Path

REPO = Path("/home/kyle/coldrawdb")
DELTA_ROOT = REPO / "logos/changes/add-baseline-docs"


def strip_md_a(content: str) -> str:
    """Strip Type A ## ADDED/MODIFIED/REMOVED + > meta block at top."""
    lines = content.split("\n")
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    s = lines[i].strip() if i < len(lines) else ""
    if not (s.startswith("## ADDED") or s.startswith("## MODIFIED") or s.startswith("## REMOVED")):
        return content
    i += 1
    while i < len(lines) and (lines[i].startswith(">") or lines[i].strip() == ""):
        i += 1
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    return "\n".join(lines[i:])


def write_delta(target_rel: str, body: str) -> None:
    tgt = DELTA_ROOT / target_rel
    tgt.parent.mkdir(parents=True, exist_ok=True)
    tgt.write_text(body, encoding="utf-8")
    print(f"  ✓ {target_rel}  ({len(body)} chars)")


# ────────────────────────────────────────────────────────────────────
# D-01-1: core-01-requirements.md — 顶部剥离 + NFR §3 增补 + §4 修正
# ────────────────────────────────────────────────────────────────────
def build_requirements() -> None:
    src = (REPO / "logos/resources/prd/1-product-requirements/core-01-requirements.md").read_text(encoding="utf-8")
    cleaned = strip_md_a(src)

    # Find the §4 范围边界 line and insert new NFR entries before it
    boundary_marker = "## 4. 范围边界（V1 不做）"
    new_nfrs = """\
| NFR-13 | 前端 WASM 体积（启用 Monaco 后） | Monaco 语言包增量 ≤ 3 MB（gzipped），按需 lazy-load，不阻塞首屏 |
| NFR-14 | 设计 token 体系 | 全部视觉属性通过 `--cdb-*` CSS 变量引用，禁止硬编码色值；token 列表见 `core-07-design-tokens.md` |
| NFR-15 | 主题切换 | 支持 light / dark 模式全局切换，token 覆盖规则见 `core-0b-dark-mode.md`；默认 light，遵循 `prefers-color-scheme` |
| NFR-16 | 动效一致性 | 模态/抽屉/按钮 hover/active 使用统一动效 token（`--cdb-duration-*` + `--cdb-easing-*`），规范见 `core-0c-motion.md` |

"""
    assert boundary_marker in cleaned, "expected §4 marker missing"
    cleaned = cleaned.replace(boundary_marker, new_nfrs + boundary_marker)

    # Update §4: remove "主题切换（V2 候选）" since now implemented
    cleaned = cleaned.replace("- ❌ 主题切换（V2 候选）\n", "")

    body = (
        "## MODIFIED — 顶部元数据剥离 + NFR 章节扩展\n\n"
        "> 模块：core | 提案：add-baseline-docs\n"
        "> 路径：`logos/resources/prd/1-product-requirements/core-01-requirements.md`\n"
        "> 策略：\n"
        "> 1. 移除文件开头 `## ADDED — V1 需求文档` + `>` 元数据块\n"
        "> 2. 在 §3 NFR 表格末尾新增 4 条（Monaco WASM 体积 / 设计 token / 主题切换 / 动效）\n"
        "> 3. 在 §4 范围边界中移除「主题切换（V2 候选）」（已在 V2 设计系统落地）\n\n"
        f"{cleaned}\n"
    )
    write_delta("deltas/prd/1-product-requirements/core-01-requirements.md", body)


# ────────────────────────────────────────────────────────────────────
# D-01-2: core-00-scenario-overview.md — 顶部剥离 + 场景↔文档映射补充
# ────────────────────────────────────────────────────────────────────
def build_scenario_overview() -> None:
    src = (REPO / "logos/resources/prd/1-product-requirements/core-00-scenario-overview.md").read_text(encoding="utf-8")
    cleaned = strip_md_a(src)

    # Add a new section §3 "功能规格索引" mapping the 16 feature specs by capability area
    extra = """

## 3. 功能规格索引（redesign phases A-E 引入 + 基线扩展）

> 覆盖范围：16 个 `core-XX-*.md` 功能规格文件（V1 基线 9 + redesign-phase-c 1 + redesign-phase-e 6）。

| 规格文件 | 阶段 | 核心能力 |
|---|---|---|
| `core-00-information-architecture.md` | V1 基线 | 顶层布局（Workspace + Modal 层级）、路由拆分 |
| `core-01-editor-canvas.md` | V1 基线 | 编辑器画布总规格、平移/缩放/框选 |
| `core-01a-table-and-field.md` | V1 基线 | 表与字段编辑（CAP-CANVAS-01/02） |
| `core-01b-relationship.md` | V1 基线 | 关系编辑（CAP-CANVAS-03） |
| `core-01c-index-enum-custom-type.md` | V1 基线 | 索引 / 枚举 / 自定义类型 |
| `core-01d-import-export.md` | redesign-phase-c | 导入 / 导出 IO 抽屉（替换 V1 模态） |
| `core-02-diagram-persistence.md` | V1 基线 | diagram CRUD + revision 乐观锁 |
| `core-03-bridge-io.md` | V1 基线 | 桥接层 7 引擎 SQL + DBML + JSON |
| `core-04-side-panel-tabs.md` | V1 基线（V2 重构） | 侧栏 7 Tab + 搜索 / 筛选 |
| `core-05-top-menu-modals.md` | V1 基线（V2 重构） | AppBar + 6 模态（New/Open/Share/Rename/Settings/Confirm） |
| `core-07-design-tokens.md` | redesign-phase-e（E1） | `--cdb-*` 设计 token 体系（13 类 ~100 token） |
| `core-08-icon-library.md` | redesign-phase-e（E2） | SVG 图标库（替代 emoji） |
| `core-09-core-components.md` | redesign-phase-e（E3） | 8 类核心组件（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet） |
| `core-0a-code-editor.md` | redesign-phase-e（E4） | Monaco 集成 + DBML setup + 复制按钮 |
| `core-0b-dark-mode.md` | redesign-phase-e（E5） | 暗色模式（`darkBgTheme = #16161A`） |
| `core-0c-motion.md` | redesign-phase-e（E6） | 动效 token + 过渡 / 微交互 |

"""
    # Insert before the "## 参考源" section if present, else append
    if "## 参考源" in cleaned:
        cleaned = cleaned.replace("## 参考源", extra.strip() + "\n\n## 参考源")
    else:
        cleaned = cleaned.rstrip() + "\n" + extra

    body = (
        "## MODIFIED — 顶部元数据剥离 + 功能规格索引章节补充\n\n"
        "> 模块：core | 提案：add-baseline-docs\n"
        "> 路径：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`\n"
        "> 策略：\n"
        "> 1. 移除文件开头 `## ADDED — 场景总览表` + `>` 元数据块\n"
        "> 2. 在原 `## 参考源` 之前新增 §3「功能规格索引」表格（16 行，覆盖 V1 基线 9 + redesign-phase-c 1 + redesign-phase-e 6）\n"
        "> 3. §1 场景索引（S01/S02/V2 计划场景）保持不变\n\n"
        f"{cleaned}\n"
    )
    write_delta("deltas/prd/1-product-requirements/core-00-scenario-overview.md", body)


# ────────────────────────────────────────────────────────────────────
# D-01-3: core-01-architecture-overview.md — 顶部剥离 + V2 布局层 + 设计系统层
# ────────────────────────────────────────────────────────────────────
def build_architecture() -> None:
    src = (REPO / "logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md").read_text(encoding="utf-8")
    cleaned = strip_md_a(src)

    # Insert V2 layout layer section between §2 (frontend) and §3 (backend)
    new_layout_section = """

## 2.5 V2 前端布局层（redesign-phase-a/b/c 落地，2026-06-14）

V2 重构后的前端分为 **4 个 UI 层**（z-index 体系见 `core-00-information-architecture.md` §1）：

| 层 | 容器 | z-index | 职责 |
|---|---|---|---|
| L1 | `AppBar`（顶栏） | `--cdb-z-app-bar` (10) | 标题 + 主菜单 + 全局操作（Share / Undo / Redo） |
| L2 | `ToolRail`（左侧工具轨） | `--cdb-z-tool-rail` (20) | 选中 / 表 / 关系 / 区域 / 便签 / 缩放 6 个工具按钮 |
| L3 | `Inspector`（右侧检查器） | `--cdb-z-inspector` (30) | 当前选中对象的属性编辑面板（替代 V1 模态中的字段编辑） |
| L4 | `ModalRoot`（模态根） | `--cdb-z-modal` (40) | New / Open / Share / Rename / Settings / Confirm 6 个模态 |
| L5 | `Palette` / `Tooltip` / `Popover` | `--cdb-z-overlay` (50) | 颜色选择器 / 工具提示 / 弹出层 |
| L6 | `Drawer`（IO 抽屉） | `--cdb-z-drawer` (35) | 导入 / 导出抽屉（与 Inspector 同级侧栏语义，不占用模态层） |

> V1 vs V2 关键差异：V1 所有编辑操作集中在中央模态（L4），V2 拆分为侧栏（L3）+ 抽屉（L6）+ 模态（L4），降低上下文切换成本。

## 2.6 设计系统层（redesign-phase-d/e 落地，2026-06-15）

| 组件 | 来源 | 引用规格 |
|---|---|---|
| Design Tokens | 13 类约 100 个 `--cdb-*` CSS 变量 | `core-07-design-tokens.md` |
| Icon Library | 自建 SVG 模板 + `@douyinfe/semi-icons` 命名规范 | `core-08-icon-library.md` |
| Core Components | 8 类（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet） | `core-09-core-components.md` |
| Code Editor | Monaco + DBML setup + 复制按钮（E4 替代 V1 `<textarea readonly>`） | `core-0a-code-editor.md` |
| Dark Mode | `<html data-mode="light\\|dark">` 全局切换（E5） | `core-0b-dark-mode.md` |
| Motion | CSS `@keyframes` + transition + 工具类（E6 不引入 framer-motion） | `core-0c-motion.md` |

> **依赖方向（强制）**：所有组件 / icon / 动效 都依赖 token 层（`core-07`），不得越过 token 直接引用硬编码值。

"""
    # Insert before "## 3. 后端 11 子模块"
    backend_marker = "## 3. 后端 11 子模块"
    assert backend_marker in cleaned
    cleaned = cleaned.replace(backend_marker, new_layout_section.strip() + "\n\n" + backend_marker)

    body = (
        "## MODIFIED — 顶部元数据剥离 + V2 布局层 + 设计系统层章节补充\n\n"
        "> 模块：core | 提案：add-baseline-docs\n"
        "> 路径：`logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md`\n"
        "> 策略：\n"
        "> 1. 移除文件开头 `## ADDED — V1 技术架构` + `>` 元数据块\n"
        "> 2. 在 §2 与 §3 之间插入 §2.5「V2 前端布局层」（6 层 z-index 体系表）\n"
        "> 3. 在 §2.5 之后插入 §2.6「设计系统层」（6 类组件 + 对应规格文件引用）\n\n"
        f"{cleaned}\n"
    )
    write_delta("deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md", body)


# ────────────────────────────────────────────────────────────────────
# D-01-4: core-baseline-reference.md — §2 模块清单扩充 + §3 Monaco 提示 + §4 归档索引
# ────────────────────────────────────────────────────────────────────
def build_baseline_reference() -> None:
    src = (REPO / "logos/resources/reference/core-baseline-reference.md").read_text(encoding="utf-8")
    # Strip leading 4 metadata lines
    lines = src.split("\n")
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    # Skip leading `# ADDED — ...` style header (this file doesn't have it, but be safe)
    cleaned = "\n".join(lines[i:])

    # Extend §2.1 frontend modules table with V2 modules + design system layer
    extra_2_1 = """| `editor_panels.rs` | TopMenuBar / Toolbar / LeftPanel / RightPanel / 模态框 |
| `editor_render.rs` | Canvas 渲染、表/字段/关系/区域/注释绘制 |
| `lib.rs` | 应用入口、pathname 解析、debug 测试钩子 |

### 2.1.1 V2 前端布局层（redesign-phase-a/b/c，2026-06-14）

| 模块 | 文件 | 职责 |
|---|---|---|
| AppBar | `frontend-rs/src/components/app_bar.rs`（E1 重构） | 顶栏（标题 + 主菜单 + 全局操作） |
| ToolRail | `frontend-rs/src/components/tool_rail.rs`（E1 重构） | 左侧工具轨（6 个工具按钮） |
| Inspector | `frontend-rs/src/components/inspector.rs`（E1 重构） | 右侧属性编辑面板 |
| ModalRoot | `frontend-rs/src/components/modal_root.rs`（E1 重构） | 6 个模态（New/Open/Share/Rename/Settings/Confirm） |
| Drawer | `frontend-rs/src/components/io_drawer.rs`（Phase C） | 导入 / 导出抽屉 |

### 2.1.2 设计系统层（redesign-phase-d/e，2026-06-15）

| 组件 | 规格文件 | 状态 |
|---|---|---|
| Design Tokens | `core-07-design-tokens.md` | E1 ✅ |
| Icon Library | `core-08-icon-library.md` | E2 ✅ |
| Core Components | `core-09-core-components.md` | E3 ✅ |
| Monaco CodeEditor | `core-0a-code-editor.md` | E4 ✅ |
| Dark Mode | `core-0b-dark-mode.md` | E5 ✅ |
| Motion | `core-0c-motion.md` | E6 ✅ |

"""
    marker_2_1 = "| `lib.rs` | 应用入口、pathname 解析、debug 测试钩子 |"
    assert marker_2_1 in cleaned
    cleaned = cleaned.replace(marker_2_1 + "\n", marker_2_1 + "\n" + extra_2_1)

    # Extend §3.1 local startup with Monaco cache note
    extra_3_1 = """

### 3.1.1 Monaco 浏览器缓存提示（E4 启用后）

启用 Monaco Editor（E4）后，前端 WASM 体积约 +3 MB（gzipped）。建议：

- 首次加载后浏览器自动缓存 `monaco-editor/*` chunk
- 后续访问不再重复下载
- 部署后第一次访问编辑器的用户首屏延迟约 +500ms（cold cache）

如需在 CI 中验证缓存生效，可用 `curl -I http://localhost:8080/monaco-editor/editor.worker.js` 检查 `Cache-Control` 头。
"""
    marker_3_1 = "```bash\n# 后端（端口 3000）\ncd backend\ncargo run\n```"
    cleaned = cleaned.replace(marker_3_1, marker_3_1 + extra_3_1)

    # Extend §4 with redesign phase archive indices
    extra_4 = """| 用途 | 路径 |
|---|---|
| 项目配置 | `logos/logos.config.json` |
| 资源索引 | `logos/logos-project.yaml` |
| AI 指令 | `AGENTS.md` |
| 历史归档索引 | `logos/changes/archive/`（含 add-frontend-completeness / redesign-phase-a~e 等 15 个已归档提案） |
| 当前活跃变更 | `logos/changes/<slug>/`（如 `add-baseline-docs`） |

### 4.1 最近归档的设计系统类变更（2026-06）

| 提案 slug | 阶段 | 关键产出 |
|---|---|---|
| `redesign-phase-a-layout` | Phase A（V2 布局） | AppBar + ToolRail + Inspector + ModalRoot 4 容器 + z-index 体系 |
| `redesign-phase-b-relationship` | Phase B | 关系工具栏 + Tooltip / Popover |
| `redesign-phase-c-import-export` | Phase C | IO 抽屉（替代 V1 Import 模态） |
| `redesign-phase-d-command-code` | Phase D | Command Palette + Code View（Phase D 已 archive，E4 Monaco 升级版生效） |
| `redesign-phase-e-design-system-migration` | Phase E | E1–E6 设计系统迁移（tokens / icons / components / Monaco / dark mode / motion） |
"""
    marker_4 = "| AI 指令 | `AGENTS.md` |"
    cleaned = cleaned.replace(marker_4, marker_4 + "\n" + extra_4)

    body = (
        "## MODIFIED — §2 模块清单扩充 + §3 Monaco 缓存提示 + §4 归档索引\n\n"
        "> 模块：core | 提案：add-baseline-docs\n"
        "> 路径：`logos/resources/reference/core-baseline-reference.md`\n"
        "> 策略：\n"
        "> 1. 在 §2.1 末尾新增 §2.1.1（V2 前端布局层 5 模块）+ §2.1.2（设计系统层 6 组件）\n"
        "> 2. 在 §3.1 末尾新增 §3.1.1（Monaco 浏览器缓存提示）\n"
        "> 3. 在 §4 表格末尾新增 2 行（历史归档索引 + 当前活跃变更）\n"
        "> 4. 新增 §4.1「最近归档的设计系统类变更」表格（5 个 redesign phase）\n\n"
        f"{cleaned}\n"
    )
    write_delta("deltas/reference/core-baseline-reference.md", body)


# ────────────────────────────────────────────────────────────────────
# D-04: core-01-deployment-plan.md — 顶部剥离 + Monaco lazy-load 缓存策略补充
# ────────────────────────────────────────────────────────────────────
def build_deployment() -> None:
    src = (REPO / "logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md").read_text(encoding="utf-8")
    cleaned = strip_md_a(src)

    # Add a new sub-section §3.4.6 "WASM 缓存策略（Monaco 启用后）" after §3.4.5
    extra_3_4 = """\
#### 3.4.5 日志与 PID

- 所有脚本日志统一输出到 `logs/` 目录
- 运行产生的 `logs/`、`*.pid` 已加入 `.gitignore`，不会被提交
- 如需排查启动失败，查看 `logs/backend.log` 与 `logs/frontend.log`

#### 3.4.6 WASM 缓存策略（Monaco 启用后，redesign-phase-e E4）

启用 Monaco Editor（`core-0a-code-editor.md`）后，前端 WASM 总体积约 +3 MB（gzipped）。为避免重复下载与首屏延迟，部署方案要求：

| 资源 | Cache-Control | 说明 |
|---|---|---|
| `*.wasm` / `editor*.js` | `public, max-age=31536000, immutable` | trunk 打包文件名带 hash，永久缓存 |
| `monaco-editor/*` chunk | `public, max-age=2592000, immutable`（30 天） | Monaco 语言包按需 lazy-load |
| `index.html` | `no-cache` | SPA 入口必须每次校验更新 |

nginx 配置示例（在 `nginx.conf` 的 `location /` 中）：

```nginx
location ~* \\.(wasm|js)$ {
  add_header Cache-Control "public, max-age=31536000, immutable";
}
location ~* /monaco-editor/ {
  add_header Cache-Control "public, max-age=2592000, immutable";
}
location = / {
  add_header Cache-Control "no-cache";
}
```

Docker 镜像层复用：`Dockerfile` 的 wasm-build 阶段产物 `dist/` 在镜像 tag 不变时复用率约 95%，多 staging 间共享层可显著降低带宽。
"""
    marker_3_4_5 = "#### 3.4.5 日志与 PID"
    assert marker_3_4_5 in cleaned
    # Insert new §3.4.6 right after the §3.4.5 block end (search for §3.4.6 marker if exists)
    if "#### 3.4.6" in cleaned:
        # Already exists somehow, just replace
        pass
    else:
        # Find §3.4.5 block end (next "#### " or "### " or "## " marker)
        idx = cleaned.index(marker_3_4_5)
        # Find the next "#### " or "## " after this position
        rest = cleaned[idx:]
        # find end of §3.4.5 — look for next "#### " heading OR end of §3 (i.e. next "## 4")
        next_section_idx = len(rest)
        for m in re.finditer(r"^#{2,4} ", rest, flags=re.MULTILINE):
            if m.start() > 0:
                next_section_idx = m.start()
                break
        # Insert before next section
        cleaned = cleaned[:idx + next_section_idx] + extra_3_4 + "\n" + cleaned[idx + next_section_idx:]

    body = (
        "## MODIFIED — 顶部元数据剥离 + §3.4.6 WASM 缓存策略章节补充\n\n"
        "> 模块：core | 提案：add-baseline-docs\n"
        "> 路径：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`\n"
        "> 策略：\n"
        "> 1. 移除文件开头 `## ADDED — V1 部署方案` + `>` 元数据块\n"
        "> 2. 在 §3.4.5 之后新增 §3.4.6「WASM 缓存策略（Monaco 启用后）」\n"
        ">   - 资源 Cache-Control 表（wasm / monaco chunk / index.html）\n"
        ">   - nginx 配置示例（按 location 区分缓存策略）\n"
        ">   - Docker 镜像层复用说明\n\n"
        f"{cleaned}\n"
    )
    write_delta("deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md", body)


import re  # needed for deployment regex


def main() -> None:
    print("Generating 5 substantive delta files under logos/changes/add-baseline-docs/deltas/")
    build_requirements()
    build_scenario_overview()
    build_architecture()
    build_baseline_reference()
    build_deployment()
    print("\nDone.")


if __name__ == "__main__":
    main()