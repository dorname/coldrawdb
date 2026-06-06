#!/bin/bash
# R-3 CI gate: verify module dependency constraints
# - editor_core must NOT import editor_render, editor_panels, or editor_data_access
# - editor_render, editor_panels, editor_data_access must each import editor_core

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="$SCRIPT_DIR/../src"

# Check if ast-grep is available
if command -v ast-grep >/dev/null 2>&1; then
    # ast-grep path: use SG to match use crate::editor_(render|panels|data_access) in editor_core.rs
    # and use SG to match use crate::editor_core in the other three
    echo "[check_module_deps] using ast-grep"

    # editor_core must NOT reference the other modules
    VIOLATIONS=$(ast-grep scan -p 'use crate::editor_(render|panels|data_access)' "$SRC_DIR/editor_core.rs" 2>/dev/null || true)
    if [ -n "$VIOLATIONS" ]; then
        echo "FAIL: editor_core.rs references other editor modules"
        echo "$VIOLATIONS"
        exit 1
    fi

    # Each sub-module must reference editor_core
    for mod in editor_render editor_panels editor_data_access; do
        REF=$(ast-grep scan -p 'use crate::editor_core' "$SRC_DIR/${mod}.rs" 2>/dev/null || true)
        if [ -z "$REF" ]; then
            echo "FAIL: ${mod}.rs does not import editor_core"
            exit 1
        fi
    done

    echo "PASS"
    exit 0
fi

# Fallback: use grep -RE
echo "[check_module_deps] using grep fallback"

# editor_core must NOT import editor_render, editor_panels, or editor_data_access
for bad in editor_render editor_panels editor_data_access; do
    if grep -RE "use crate::$bad" "$SRC_DIR/editor_core.rs" >/dev/null 2>&1; then
        echo "FAIL: editor_core.rs references $bad"
        exit 1
    fi
done

# editor_render, editor_panels, editor_data_access must each import editor_core
for mod in editor_render editor_panels editor_data_access; do
    if ! grep -RE "use crate::editor_core" "$SRC_DIR/${mod}.rs" >/dev/null 2>&1; then
        echo "FAIL: ${mod}.rs does not import editor_core"
        exit 1
    fi
done

echo "PASS"
exit 0