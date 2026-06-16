#!/usr/bin/env python3
"""Batch-generate mechanical "strip top metadata" delta files.

Run from repo root: python3 logos/changes/add-baseline-docs/scripts/build-deltas.py

Handles three patterns:
- Type A: `## ADDED — title` + `> meta` block (most markdown files)
- Type B: `# Delta — xxx（新文件）` wrapper + `## ADDED — 全文` + `> meta` (redesign-phase-e new files)
- Type C: `# Real Title` followed by `> meta` block (e.g. core-01d-import-export.md)

For non-md files:
- YAML: strip `# ADDED` + meta comment lines
- SQL: strip `-- ADDED` + meta comment lines
- JSON: remove the `"proposal": "add-baseline-docs"` field
"""
from __future__ import annotations
import re
from pathlib import Path

REPO = Path("/home/kyle/coldrawdb")
DELTA_ROOT = REPO / "logos/changes/add-baseline-docs"  # tgt_rel starts with "deltas/" so it resolves correctly

# (source_rel, delta_rel, kind)
# kind: "md-a" | "md-b" | "md-c" | "md-d" | "yaml" | "sql" | "json"
PAIRS = [
    # ── D-02 功能规格（16 文件）──
    ("logos/resources/prd/2-product-design/1-feature-specs/core-00-information-architecture.md",
     "deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md",
     "deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md",
     "deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md",
     "deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md",
     "deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-01d-import-export.md",
     "deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md", "md-c"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md",
     "deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-03-bridge-io.md",
     "deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md",
     "deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md",
     "deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md", "md-a"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-07-design-tokens.md",
     "deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md", "md-b"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-08-icon-library.md",
     "deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md", "md-b"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-09-core-components.md",
     "deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md", "md-b"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-0a-code-editor.md",
     "deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md", "md-b"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md",
     "deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md", "md-b"),
    ("logos/resources/prd/2-product-design/1-feature-specs/core-0c-motion.md",
     "deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md", "md-b"),
    # ── D-03 场景与测试（5 文件）──
    ("logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md",
     "deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md", "md-a"),
    ("logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md",
     "deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md", "md-a"),
    ("logos/resources/test/core-S01-test-cases.md",
     "deltas/test/core-S01-test-cases.md", "md-d"),
    ("logos/resources/test/core-S02-test-cases.md",
     "deltas/test/core-S02-test-cases.md", "md-d"),
    ("logos/resources/test/smoke/core-smoke-test-cases.md",
     "deltas/test/smoke/core-smoke-test-cases.md", "md-d"),
    # ── D-04 部署方案与实现清单（1 文件，deployment 由手工撰写，implementation 是机械）──
    ("logos/resources/implementation/core-implementation-checklist.md",
     "deltas/implementation/core-implementation-checklist.md", "md-d"),
    # ── D-05 API / DB / Scenario（5 文件）──
    ("logos/resources/api/bridge.yaml",
     "deltas/api/bridge.yaml", "yaml"),
    ("logos/resources/api/diagrams.yaml",
     "deltas/api/diagrams.yaml", "yaml"),
    ("logos/resources/database/coldrawdb-v1.sql",
     "deltas/database/coldrawdb-v1.sql", "sql"),
    ("logos/resources/scenario/core-S01-diagram-save.json",
     "deltas/scenario/core-S01-diagram-save.json", "json"),
    ("logos/resources/scenario/core-S02-shared-link-load.json",
     "deltas/scenario/core-S02-shared-link-load.json", "json"),
]


def is_delta_marker(line: str) -> bool:
    s = line.strip()
    return (
        s.startswith("## ADDED")
        or s.startswith("## MODIFIED")
        or s.startswith("## REMOVED")
    )


def is_delta_wrapper(line: str) -> bool:
    s = line.strip()
    return s.startswith("# Delta —") or s.startswith("# ADDED") or s.startswith("# MODIFIED") or s.startswith("# REMOVED")


def strip_md_a(content: str) -> str:
    """Type A: ## ADDED/MODIFIED/REMOVED + > meta block at top."""
    lines = content.split("\n")
    i = 0
    # Skip leading blanks
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    if not is_delta_marker(lines[i] if i < len(lines) else ""):
        return content
    i += 1
    # Skip blockquote + blanks
    while i < len(lines) and (lines[i].startswith(">") or lines[i].strip() == ""):
        i += 1
    # Skip any leading blank lines after metadata
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    return "\n".join(lines[i:])


def strip_md_b(content: str) -> str:
    """Type B: # Delta — xxx（新文件）+ > + ## ADDED — 全文 + > + ... + # Real Title."""
    lines = content.split("\n")
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    # Skip `# Delta — ...` line
    if i < len(lines) and lines[i].strip().startswith("# Delta —"):
        i += 1
        while i < len(lines) and (lines[i].startswith(">") or lines[i].strip() == ""):
            i += 1
        # Skip optional `## ADDED — 全文` + blockquote
        if i < len(lines) and is_delta_marker(lines[i]):
            i += 1
            while i < len(lines) and (lines[i].startswith(">") or lines[i].strip() == ""):
                i += 1
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    return "\n".join(lines[i:])


def strip_md_c(content: str) -> str:
    """Type C: # Real Title + (blank) + > meta block + (blank) + content. Keep H1, strip meta."""
    lines = content.split("\n")
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    if i >= len(lines) or not lines[i].startswith("# "):
        return content
    h1_idx = i  # save H1 position
    # Look ahead for blockquote block
    j = i + 1
    while j < len(lines) and lines[j].strip() == "":
        j += 1
    if j < len(lines) and lines[j].startswith(">"):
        # Strip everything from H1+1 to end of blockquote; keep H1
        k = j
        while k < len(lines) and lines[k].startswith(">"):
            k += 1
        # Skip trailing blanks
        while k < len(lines) and lines[k].strip() == "":
            k += 1
        return lines[h1_idx] + "\n\n" + "\n".join(lines[k:])
    return content


def strip_md_d(content: str) -> str:
    """Type D: # ADDED — xxx + # meta lines (used by test cases + implementation checklist)."""
    lines = content.split("\n")
    i = 0
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    if i < len(lines) and lines[i].strip().startswith("# ADDED"):
        i += 1
        # Skip following `# ` lines + blank lines
        while i < len(lines) and (lines[i].startswith("# ") or lines[i].strip() == ""):
            i += 1
    while i < len(lines) and lines[i].strip() == "":
        i += 1
    return "\n".join(lines[i:])


def strip_md(content: str, kind: str) -> str:
    if kind == "md-a":
        return strip_md_a(content)
    if kind == "md-b":
        return strip_md_b(content)
    if kind == "md-c":
        return strip_md_c(content)
    if kind == "md-d":
        return strip_md_d(content)
    raise ValueError(f"Unknown md kind: {kind}")


def strip_yaml(content: str) -> str:
    """Strip leading `# ADDED — ...` + `# meta` lines."""
    lines = content.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        s = line.strip()
        if s.startswith("# ADDED") or s.startswith("# MODIFIED") or s.startswith("# REMOVED"):
            i += 1
            continue
        if line.startswith("#") and any(
            marker in line
            for marker in ("提案：add-baseline-docs", "模块：core", "路径：", "对齐参考源")
        ):
            i += 1
            continue
        if s == "":
            i += 1
            continue
        break
    return "\n".join(lines[i:])


def strip_sql(content: str) -> str:
    """Strip leading `-- ADDED — ...` + `-- meta` lines."""
    lines = content.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        s = line.strip()
        if s.startswith("-- ADDED") or s.startswith("-- MODIFIED") or s.startswith("-- REMOVED"):
            i += 1
            continue
        if line.startswith("--") and any(
            marker in line
            for marker in ("提案：add-baseline-docs", "模块：core", "路径：", "对齐参考源")
        ):
            i += 1
            continue
        if s == "":
            i += 1
            continue
        break
    return "\n".join(lines[i:])


def strip_json(content: str) -> str:
    """Remove the `"proposal": "add-baseline-docs"` field line(s)."""
    lines = content.split("\n")
    out = []
    for line in lines:
        if re.match(r'^\s*"proposal":\s*"add-baseline-docs"\s*,?\s*$', line):
            continue
        out.append(line)
    return "\n".join(out)


def make_md_delta(cleaned: str, source_rel: str, kind: str) -> str:
    if kind == "md-a":
        strategy = (
            "移除文件开头的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` "
            "标记块及其紧随的 `>` 元数据行，保留正文首个一级标题以下所有内容原样。"
        )
    elif kind == "md-b":
        strategy = (
            "移除文件开头的 `# Delta — xxx（新文件）` 包装块与紧随的 `## ADDED — 全文` "
            "子块及其 `>` 元数据行，保留真实一级标题以下所有内容原样。"
        )
    elif kind == "md-c":
        strategy = (
            "移除文件一级标题（`# Real Title`）紧随的 `> 模块：core | 提案：xxx` 元数据块，"
            "保留一级标题与正文以下所有内容原样。"
        )
    else:  # md-d
        strategy = (
            "移除文件开头的 `# ADDED — ...` 单井号标题与紧随的 `#` 元数据行（提案 / 模块 / 路径 / 对齐参考源），"
            "保留第一个 `##` 二级标题以下所有内容原样。"
        )
    return (
        f"## MODIFIED — 顶部元数据剥离\n\n"
        f"> 模块：core | 提案：add-baseline-docs\n"
        f"> 路径：`{source_rel}`\n"
        f"> 策略：{strategy}\n\n"
        f"{cleaned}\n"
    )


def make_yaml_delta(cleaned: str, source_rel: str) -> str:
    return (
        "## MODIFIED — 顶部元数据剥离\n\n"
        f"> 模块：core | 提案：add-baseline-docs\n"
        f"> 路径：`{source_rel}`\n"
        f"> 策略：移除文件开头的 `# ADDED — ...` 与紧随的 `#` 元数据注释行（提案 / 模块 / 路径 / 对齐参考源），"
        f"保留 `openapi:` 以下全部 endpoints 定义不变。\n\n"
        f"{cleaned}\n"
    )


def make_sql_delta(cleaned: str, source_rel: str) -> str:
    return (
        "## MODIFIED — 顶部元数据剥离\n\n"
        f"> 模块：core | 提案：add-baseline-docs\n"
        f"> 路径：`{source_rel}`\n"
        f"> 策略：移除文件开头的 `-- ADDED — ...` 与紧随的 `--` 元数据注释行（提案 / 模块 / 路径 / 对齐参考源），"
        f"保留全部 DDL 不变。\n\n"
        f"{cleaned}\n"
    )


def make_json_delta(cleaned: str, source_rel: str) -> str:
    return (
        "## MODIFIED — 顶部元数据剥离\n\n"
        f"> 模块：core | 提案：add-baseline-docs\n"
        f"> 路径：`{source_rel}`\n"
        f"> 策略：移除 JSON 体内 `\"proposal\": \"add-baseline-docs\"` 字段，"
        f"保留 `scenario_id` / `steps` / `preconditions` 等全部业务字段不变。\n\n"
        f"{cleaned}\n"
    )


def process(src_rel: str, tgt_rel: str, kind: str) -> None:
    src = REPO / src_rel
    tgt = DELTA_ROOT / tgt_rel
    content = src.read_text(encoding="utf-8")

    if kind.startswith("md-"):
        cleaned = strip_md(content, kind)
        delta = make_md_delta(cleaned, src_rel, kind)
    elif kind == "yaml":
        cleaned = strip_yaml(content)
        delta = make_yaml_delta(cleaned, src_rel)
    elif kind == "sql":
        cleaned = strip_sql(content)
        delta = make_sql_delta(cleaned, src_rel)
    elif kind == "json":
        cleaned = strip_json(content)
        delta = make_json_delta(cleaned, src_rel)
    else:
        raise ValueError(f"Unknown kind: {kind}")

    tgt.parent.mkdir(parents=True, exist_ok=True)
    tgt.write_text(delta, encoding="utf-8")
    print(f"  ✓ {tgt_rel}  ({len(content)} → {len(cleaned)} chars)")


def main() -> None:
    print(f"Generating {len(PAIRS)} mechanical delta files under {DELTA_ROOT.relative_to(REPO)}")
    for src_rel, tgt_rel, kind in PAIRS:
        process(src_rel, tgt_rel, kind)
    print(f"\nDone. Generated {len(PAIRS)} delta files.")


if __name__ == "__main__":
    main()