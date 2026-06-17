#!/usr/bin/env python3
"""Execute MERGE_PROMPT.md: merge all 32 delta files into logos/resources/.

For each delta file, extract the content from the `## MODIFIED — ...` block
(removing the proposal metadata header) and overwrite the target main document.

Run from repo root: python3 logos/changes/add-baseline-docs/scripts/merge-exec.py
"""
from __future__ import annotations
import re
from pathlib import Path

REPO = Path("/home/kyle/coldrawdb")
DELTA_BASE = REPO / "logos/changes/add-baseline-docs/deltas"
RESOURCES = REPO / "logos/resources"


def extract_cleaned_content(delta_text: str) -> str:
    """Find the `## MODIFIED/ADDED/REMOVED — ...` block and return content after the meta `>` lines."""
    # Find the first H2 marker
    m = re.search(r"^## (MODIFIED|ADDED|REMOVED) — .+$", delta_text, flags=re.MULTILINE)
    if not m:
        raise ValueError("No MODIFIED/ADDED/REMOVED marker found in delta")
    # Walk past the H2 line
    i = m.end() + 1  # +1 for newline
    # Skip blank lines
    while i < len(delta_text) and delta_text[i] == "\n":
        i += 1
    # Skip `> ...` meta lines + blank lines
    while i < len(delta_text):
        if delta_text[i] == ">":
            # Skip to end of line
            nl = delta_text.find("\n", i)
            if nl == -1:
                i = len(delta_text)
            else:
                i = nl + 1
        elif delta_text[i] == "\n":
            # blank line — peek ahead: if next non-blank is `>` continue, else break
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


def merge_one(delta_path: Path) -> None:
    """Merge one delta file to its target resource."""
    rel = delta_path.relative_to(DELTA_BASE)
    target = RESOURCES / rel
    delta_text = delta_path.read_text(encoding="utf-8")
    cleaned = extract_cleaned_content(delta_text)

    # Ensure target dir exists
    target.parent.mkdir(parents=True, exist_ok=True)

    # Backup the original (just for safety, not committed)
    if target.exists():
        backup = target.with_suffix(target.suffix + ".bak")
        if not backup.exists():
            backup.write_text(target.read_text(encoding="utf-8"), encoding="utf-8")

    target.write_text(cleaned, encoding="utf-8")
    print(f"  ✓ {rel}  (delta {len(delta_text)} → target {len(cleaned)} chars)")


def main() -> None:
    deltas = sorted(DELTA_BASE.rglob("*"))
    files = [d for d in deltas if d.is_file()]
    print(f"Merging {len(files)} delta files from {DELTA_BASE.relative_to(REPO)} to {RESOURCES.relative_to(REPO)}/")
    for delta in files:
        merge_one(delta)
    print(f"\nDone. Merged {len(files)} delta files.")


if __name__ == "__main__":
    main()