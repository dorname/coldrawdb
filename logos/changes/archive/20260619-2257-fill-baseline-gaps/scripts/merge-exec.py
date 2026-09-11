#!/usr/bin/env python3
"""Execute MERGE_PROMPT.md: merge all 6 delta files into logos/ resources.

For each delta file, extract the content from the `## MODIFIED — ...` block
(removing the proposal metadata header) and overwrite the target main document.

Special handling for logos-project.yaml: target is logos/logos-project.yaml
(not under logos/resources/).

Run from repo root: python3 logos/changes/fill-baseline-gaps/scripts/merge-exec.py
"""
from __future__ import annotations
import re
from pathlib import Path

REPO = Path("/home/kyle/coldrawdb")
DELTA_BASE = REPO / "logos/changes/fill-baseline-gaps/deltas"
RESOURCES = REPO / "logos/resources"
LOGOS_ROOT = REPO / "logos"


def extract_cleaned_content(delta_text: str) -> str:
    """Find the `## MODIFIED/ADDED/REMOVED — ...` block and return content after the meta `>` lines."""
    m = re.search(r"^## (MODIFIED|ADDED|REMOVED) — .+$", delta_text, flags=re.MULTILINE)
    if not m:
        raise ValueError("No MODIFIED/ADDED/REMOVED marker found in delta")
    i = m.end() + 1
    while i < len(delta_text) and delta_text[i] == "\n":
        i += 1
    while i < len(delta_text):
        if delta_text[i] == ">":
            nl = delta_text.find("\n", i)
            i = nl + 1 if nl != -1 else len(delta_text)
        elif delta_text[i] == "\n":
            j = i
            while j < len(delta_text) and delta_text[j] == "\n":
                j += 1
            if j < len(delta_text) and delta_text[j] == ">":
                i = j
                continue
            else:
                i += 1
        else:
            break
    return delta_text[i:]


def merge_one(delta_path: Path, target: Path) -> None:
    """Merge one delta file to its explicit target path."""
    delta_text = delta_path.read_text(encoding="utf-8")
    cleaned = extract_cleaned_content(delta_text)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        backup = target.with_suffix(target.suffix + ".bak")
        if not backup.exists():
            backup.write_text(target.read_text(encoding="utf-8"), encoding="utf-8")
    target.write_text(cleaned, encoding="utf-8")
    print(f"  ✓ {delta_path.relative_to(REPO)}  →  {target.relative_to(REPO)}  (delta {len(delta_text)} → target {len(cleaned)} chars)")


def main() -> None:
    print("Merging 6 delta files from logos/changes/fill-baseline-gaps/deltas/ ...")
    print()

    # 5 PRD deltas: standard RESOURCES / rel mapping (DELTA_BASE already ends with deltas/)
    prd_deltas = [
        "prd/1-product-requirements/core-01-requirements.md",
        "prd/1-product-requirements/core-02-product-vision.md",
        "prd/1-product-requirements/core-03-pain-points.md",
        "prd/1-product-requirements/core-04-scenario-detail.md",
        "prd/3-technical-plan/1-architecture/core-01-architecture-overview.md",
    ]
    for rel in prd_deltas:
        delta_path = DELTA_BASE / rel
        target = RESOURCES / rel
        merge_one(delta_path, target)

    # 1 special: logos-project.yaml → logos/logos-project.yaml (not under resources/)
    special_delta = DELTA_BASE / "logos-project.yaml"
    special_target = LOGOS_ROOT / "logos-project.yaml"
    merge_one(special_delta, special_target)

    print()
    print("Done. Merged 6 delta files.")


if __name__ == "__main__":
    main()