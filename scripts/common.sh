#!/usr/bin/env bash
# Common utilities for local run scripts.
# shellcheck disable=SC2155

set -euo pipefail

# Project layout
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Default configuration
export COLDRAWDB_BACKEND_PORT="${COLDRAWDB_BACKEND_PORT:-3000}"
export COLDRAWDB_FRONTEND_PORT="${COLDRAWDB_FRONTEND_PORT:-8080}"
export COLDRAWDB_BACKEND_LOG="${COLDRAWDB_BACKEND_LOG:-logs/backend.log}"
export COLDRAWDB_FRONTEND_LOG="${COLDRAWDB_FRONTEND_LOG:-logs/frontend.log}"
export COLDRAWDB_BACKEND_PID="${COLDRAWDB_BACKEND_PID:-logs/backend.pid}"
export COLDRAWDB_FRONTEND_PID="${COLDRAWDB_FRONTEND_PID:-logs/frontend.pid}"
export COLDRAWDB_HEALTH_TIMEOUT="${COLDRAWDB_HEALTH_TIMEOUT:-60}"

log_info() {
    echo "[INFO] $*"
}

log_error() {
    echo "[ERROR] $*" >&2
}

log_warn() {
    echo "[WARN] $*" >&2
}

ensure_logs_dir() {
    mkdir -p "${REPO_ROOT}/logs"
}

# Resolve absolute paths relative to REPO_ROOT
abs_path() {
    local path="$1"
    if [[ "$path" = /* ]]; then
        echo "$path"
    else
        echo "${REPO_ROOT}/${path}"
    fi
}

backend_log() { abs_path "$COLDRAWDB_BACKEND_LOG"; }
frontend_log() { abs_path "$COLDRAWDB_FRONTEND_LOG"; }
backend_pid_file() { abs_path "$COLDRAWDB_BACKEND_PID"; }
frontend_pid_file() { abs_path "$COLDRAWDB_FRONTEND_PID"; }

port_in_use() {
    local port="$1"
    if command -v ss >/dev/null 2>&1; then
        ss -tln 2>/dev/null | awk -v p="$port" '$4 ~ ":"p"$" {found=1} END {exit !found}'
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tln 2>/dev/null | awk -v p="$port" '$4 ~ "."p"$" {found=1} END {exit !found}'
    else
        # Fallback: try to connect
        (echo >/dev/tcp/127.0.0.1/"$port") >/dev/null 2>&1
    fi
}

check_dependency() {
    local cmd="$1"
    local hint="${2:-}"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        log_error "Missing dependency: $cmd"
        [[ -n "$hint" ]] && log_error "Hint: $hint"
        return 1
    fi
}

check_wasm_target() {
    if ! rustup target list --installed 2>/dev/null | grep -q 'wasm32-unknown-unknown'; then
        log_error "wasm32-unknown-unknown target is not installed."
        log_error "Run: rustup target add wasm32-unknown-unknown"
        return 1
    fi
}

wait_for_http() {
    local url="$1"
    local timeout="${2:-30}"
    local deadline=$((SECONDS + timeout))
    while [ $SECONDS -lt $deadline ]; do
        if curl -fsS "$url" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    return 1
}

read_pid() {
    local file="$1"
    if [[ -f "$file" ]]; then
        cat "$file"
    fi
}

is_process_alive() {
    local pid="$1"
    [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

stop_process() {
    local pid="$1"
    local name="$2"
    if ! is_process_alive "$pid"; then
        log_info "$name (pid $pid) is not running."
        return 0
    fi

    log_info "Stopping $name (pid $pid) ..."
    kill -TERM "$pid" 2>/dev/null || true

    local waited=0
    while is_process_alive "$pid" && [ $waited -lt 10 ]; do
        sleep 1
        waited=$((waited + 1))
    done

    if is_process_alive "$pid"; then
        log_warn "$name did not stop gracefully, sending KILL"
        kill -KILL "$pid" 2>/dev/null || true
        sleep 1
    fi

    if is_process_alive "$pid"; then
        log_error "Failed to stop $name (pid $pid)"
        return 1
    fi

    log_info "$name stopped."
    return 0
}
