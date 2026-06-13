#!/usr/bin/env bash
# Start the local DrawDB backend and frontend.
# Usage: ./scripts/start-local.sh

set -euo pipefail

source "$(dirname "$0")/common.sh"

main() {
    cd "$REPO_ROOT"

    log_info "Starting DrawDB local services ..."
    log_info "Backend port: $COLDRAWDB_BACKEND_PORT"
    log_info "Frontend port: $COLDRAWDB_FRONTEND_PORT"

    # Dependency checks
    check_dependency cargo "https://rustup.rs/"
    check_dependency trunk "cargo install --locked trunk"
    check_dependency curl "install curl"
    check_wasm_target

    # Port checks
    if port_in_use "$COLDRAWDB_BACKEND_PORT"; then
        log_error "Port $COLDRAWDB_BACKEND_PORT is already in use."
        log_error "Set COLDRAWDB_BACKEND_PORT to use a different port."
        exit 1
    fi

    if port_in_use "$COLDRAWDB_FRONTEND_PORT"; then
        log_error "Port $COLDRAWDB_FRONTEND_PORT is already in use."
        log_error "Set COLDRAWDB_FRONTEND_PORT to use a different port."
        exit 1
    fi

    ensure_logs_dir

    local backend_log backend_pid frontend_log frontend_pid
    backend_log="$(backend_log)"
    backend_pid="$(backend_pid_file)"
    frontend_log="$(frontend_log)"
    frontend_pid="$(frontend_pid_file)"

    # Clean up stale PID files
    rm -f "$backend_pid" "$frontend_pid"

    # Start backend
    log_info "Starting backend (cargo run --release) ..."
    (
        cd backend
        exec cargo run --release
    ) >> "$backend_log" 2>&1 &
    local backend_pid_value=$!
    echo "$backend_pid_value" > "$backend_pid"
    log_info "Backend started with pid $backend_pid_value. Logs: $backend_log"

    # Wait for backend to be healthy
    log_info "Waiting for backend to be ready (timeout: ${COLDRAWDB_HEALTH_TIMEOUT}s) ..."
    if ! wait_for_http "http://127.0.0.1:${COLDRAWDB_BACKEND_PORT}/" "$COLDRAWDB_HEALTH_TIMEOUT"; then
        log_error "Backend failed to start within ${COLDRAWDB_HEALTH_TIMEOUT}s."
        log_error "Check logs: $backend_log"
        stop_process "$backend_pid_value" "backend" || true
        rm -f "$backend_pid"
        exit 1
    fi
    log_info "Backend is ready."

    # Start frontend
    log_info "Starting frontend (trunk serve --port $COLDRAWDB_FRONTEND_PORT) ..."
    (
        cd frontend-rs
        exec trunk serve --port "$COLDRAWDB_FRONTEND_PORT"
    ) >> "$frontend_log" 2>&1 &
    local frontend_pid_value=$!
    echo "$frontend_pid_value" > "$frontend_pid"
    log_info "Frontend started with pid $frontend_pid_value. Logs: $frontend_log"

    # Wait briefly for frontend to bind
    log_info "Waiting for frontend to be ready (timeout: 30s) ..."
    if ! wait_for_http "http://127.0.0.1:${COLDRAWDB_FRONTEND_PORT}/" 30; then
        log_error "Frontend failed to start within 30s."
        log_error "Check logs: $frontend_log"
        stop_process "$frontend_pid_value" "frontend" || true
        rm -f "$frontend_pid"
        stop_process "$backend_pid_value" "backend" || true
        rm -f "$backend_pid"
        exit 1
    fi
    log_info "Frontend is ready."

    log_info "Services started successfully."
    log_info "Backend: http://127.0.0.1:${COLDRAWDB_BACKEND_PORT}/"
    log_info "Frontend: http://127.0.0.1:${COLDRAWDB_FRONTEND_PORT}/editor"
    log_info "Stop with: ./scripts/stop-local.sh"
}

main "$@"
