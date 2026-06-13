#!/usr/bin/env bash
# Stop the local DrawDB backend and frontend started by start-local.sh.
# Usage: ./scripts/stop-local.sh

set -euo pipefail

source "$(dirname "$0")/common.sh"

main() {
    cd "$REPO_ROOT"

    log_info "Stopping DrawDB local services ..."

    local frontend_pid backend_pid
    frontend_pid="$(read_pid "$(frontend_pid_file)")"
    backend_pid="$(read_pid "$(backend_pid_file)")"

    local failed=0

    # Stop frontend first, then backend
    if [[ -n "$frontend_pid" ]]; then
        if ! stop_process "$frontend_pid" "frontend"; then
            failed=1
        fi
        rm -f "$(frontend_pid_file)"
    else
        log_warn "No frontend PID file found."
    fi

    if [[ -n "$backend_pid" ]]; then
        if ! stop_process "$backend_pid" "backend"; then
            failed=1
        fi
        rm -f "$(backend_pid_file)"
    else
        log_warn "No backend PID file found."
    fi

    if [ $failed -ne 0 ]; then
        log_error "Some services could not be stopped cleanly."
        exit 1
    fi

    log_info "All services stopped."
}

main "$@"
