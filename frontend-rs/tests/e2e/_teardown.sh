#!/bin/bash
set -e

if [ -f /tmp/e2e_pids ]; then
  BACKEND_PID=$(sed -n '1p' /tmp/e2e_pids | cut -d= -f2)
  TRUNK_PID=$(sed -n '2p' /tmp/e2e_pids | cut -d= -f2)
  kill $TRUNK_PID 2>/dev/null || true
  kill $BACKEND_PID 2>/dev/null || true
  rm -f /tmp/e2e_pids
fi

echo "Teardown complete"