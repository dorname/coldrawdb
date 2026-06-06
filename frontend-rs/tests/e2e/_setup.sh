#!/bin/bash
set -e

cd /home/kyle/coldrawdb/backend
cargo build --release
./target/release/backend &
BACKEND_PID=$!

cd /home/kyle/coldrawdb/frontend-rs
trunk serve --port 8080 &
TRUNK_PID=$!

sleep 5

echo "backend_pid=$BACKEND_PID" > /tmp/e2e_pids
echo "trunk_pid=$TRUNK_PID" >> /tmp/e2e_pids
echo "Setup complete: backend=$BACKEND_PID trunk=$TRUNK_PID"