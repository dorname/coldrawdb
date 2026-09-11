#!/usr/bin/env bash
# Integration test for local start/stop scripts.
# Uses custom ports to avoid colliding with a developer's normal session.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
START_SCRIPT="${REPO_ROOT}/scripts/start-local.sh"
STOP_SCRIPT="${REPO_ROOT}/scripts/stop-local.sh"

# Backend port is fixed in backend/config.toml (COLDRAWDB_BACKEND_PORT only affects health checks).
TEST_BACKEND_PORT=3000
TEST_FRONTEND_PORT=18080
unset COLDRAWDB_BACKEND_PORT
# persist-public-base-url：保证默认注入用例不受外部环境污染
unset PUBLIC_BASE_URL
export COLDRAWDB_FRONTEND_PORT=$TEST_FRONTEND_PORT
export COLDRAWDB_BACKEND_LOG="logs/test-backend.log"
export COLDRAWDB_FRONTEND_LOG="logs/test-frontend.log"
export COLDRAWDB_BACKEND_PID="logs/test-backend.pid"
export COLDRAWDB_FRONTEND_PID="logs/test-frontend.pid"
export COLDRAWDB_HEALTH_TIMEOUT=120

fail() {
    echo "[FAIL] $*" >&2
    exit 1
}

info() {
    echo "[INFO] $*"
}

cleanup() {
    info "Cleaning up ..."
    if [[ -f "${REPO_ROOT}/logs/test-backend.pid" ]]; then
        local pid
        pid="$(cat "${REPO_ROOT}/logs/test-backend.pid")"
        kill -TERM "$pid" 2>/dev/null || true
    fi
    if [[ -f "${REPO_ROOT}/logs/test-frontend.pid" ]]; then
        local pid
        pid="$(cat "${REPO_ROOT}/logs/test-frontend.pid")"
        kill -TERM "$pid" 2>/dev/null || true
    fi
    sleep 2
}

trap cleanup EXIT

port_open() {
    local port="$1"
    if command -v ss >/dev/null 2>&1; then
        ss -tln 2>/dev/null | awk -v p="$port" '$4 ~ ":"p"$" {found=1} END {exit !found}'
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tln 2>/dev/null | awk -v p="$port" '$4 ~ "."p"$" {found=1} END {exit !found}'
    else
        (echo >/dev/tcp/127.0.0.1/"$port") >/dev/null 2>&1
    fi
}

main() {
    cd "$REPO_ROOT"

    info "Testing local start/stop scripts with ports $TEST_BACKEND_PORT/$TEST_FRONTEND_PORT"

    # persist-public-base-url：PUBLIC_BASE_URL 默认值注入（common.sh 纯逻辑，不起进程）
    info "Checking PUBLIC_BASE_URL default injection ..."
    local default_url
    default_url="$(env -u PUBLIC_BASE_URL -u COLDRAWDB_FRONTEND_PORT bash -c 'source scripts/common.sh; echo "$PUBLIC_BASE_URL"')"
    if [[ "$default_url" != "http://127.0.0.1:8080" ]]; then
        fail "PUBLIC_BASE_URL default should be http://127.0.0.1:8080, got: $default_url"
    fi

    # persist-public-base-url：显式设置优先于默认值
    info "Checking PUBLIC_BASE_URL override ..."
    local override_url
    override_url="$(PUBLIC_BASE_URL=http://192.168.1.10:8080 bash -c 'source scripts/common.sh; echo "$PUBLIC_BASE_URL"')"
    if [[ "$override_url" != "http://192.168.1.10:8080" ]]; then
        fail "PUBLIC_BASE_URL override should win, got: $override_url"
    fi

    # Pre-condition: ports must be free
    if port_open "$TEST_BACKEND_PORT"; then
        fail "Port $TEST_BACKEND_PORT is already in use before test."
    fi
    if port_open "$TEST_FRONTEND_PORT"; then
        fail "Port $TEST_FRONTEND_PORT is already in use before test."
    fi

    # Run start script
    info "Running start-local.sh ..."
    if ! "$START_SCRIPT"; then
        fail "start-local.sh exited with non-zero status."
    fi

    # Verify PID files exist
    [[ -f "logs/test-backend.pid" ]] || fail "Backend PID file missing"
    [[ -f "logs/test-frontend.pid" ]] || fail "Frontend PID file missing"

    # Verify processes are alive
    local backend_pid frontend_pid
    backend_pid="$(cat logs/test-backend.pid)"
    frontend_pid="$(cat logs/test-frontend.pid)"
    kill -0 "$backend_pid" || fail "Backend process $backend_pid is not alive"
    kill -0 "$frontend_pid" || fail "Frontend process $frontend_pid is not alive"

    # persist-public-base-url：后端进程实际收到默认注入的 PUBLIC_BASE_URL（指向测试前端端口）
    info "Checking backend process PUBLIC_BASE_URL env ..."
    local backend_env_url
    backend_env_url="$(tr '\0' '\n' < "/proc/$backend_pid/environ" | grep '^PUBLIC_BASE_URL=' | cut -d= -f2- || true)"
    if [[ "$backend_env_url" != "http://127.0.0.1:${TEST_FRONTEND_PORT}" ]]; then
        fail "Backend env PUBLIC_BASE_URL should be http://127.0.0.1:${TEST_FRONTEND_PORT}, got: $backend_env_url"
    fi

    # Verify HTTP endpoints
    info "Checking backend endpoint ..."
    if ! curl -fsS "http://127.0.0.1:${TEST_BACKEND_PORT}/" >/dev/null; then
        fail "Backend endpoint is not reachable."
    fi

    info "Checking frontend endpoint ..."
    local frontend_body
    frontend_body="$(curl -fsS "http://127.0.0.1:${TEST_FRONTEND_PORT}/" || true)"
    if [[ -z "$frontend_body" ]]; then
        fail "Frontend endpoint is not reachable."
    fi
    if [[ "$frontend_body" != *'id="app"'* ]]; then
        fail "Frontend response does not contain expected 'app' mount element."
    fi

    # Run stop script
    info "Running stop-local.sh ..."
    if ! "$STOP_SCRIPT"; then
        fail "stop-local.sh exited with non-zero status."
    fi

    # Verify PID files removed
    [[ ! -f "logs/test-backend.pid" ]] || fail "Backend PID file still exists after stop"
    [[ ! -f "logs/test-frontend.pid" ]] || fail "Frontend PID file still exists after stop"

    # Verify processes are gone
    sleep 2
    if kill -0 "$backend_pid" 2>/dev/null; then
        fail "Backend process $backend_pid is still running after stop."
    fi
    if kill -0 "$frontend_pid" 2>/dev/null; then
        fail "Frontend process $frontend_pid is still running after stop."
    fi

    # Verify ports are free
    if port_open "$TEST_BACKEND_PORT"; then
        fail "Port $TEST_BACKEND_PORT is still open after stop."
    fi
    if port_open "$TEST_FRONTEND_PORT"; then
        fail "Port $TEST_FRONTEND_PORT is still open after stop."
    fi

    info "All local script tests passed."
}

main "$@"
