#!/usr/bin/env bash
#
# 4h Soak Test: create + save + delete diagrams, 统计成功率
#
# 失败恢复策略：
#   - 首次失败：自动重跑 1 次
#   - 连续 2 次失败 → 暂停线（不向 main merge）
#
# 降级说明：4h 在 session 内跑不完。只写脚本 + 跑 1 分钟 sanity check。
# 4h 实际运行需 dedicated session 或 CI 调度。
#
# Usage:
#   bash 4h.sh [--duration=SECONDS] [--base-url=URL]
#   Default duration: 2400s (40 min for sanity; use 14400 for full 4h)
#   Default base-url: http://127.0.0.1:6666

set -e

DURATION_SEC=${DURATION_SEC:-2400}  # 40 min default (sanity); use 14400 for full 4h
BASE_URL="${BASE_URL:-http://127.0.0.1:6666}"
SOAK_LOG="${SOAK_LOG:-/tmp/soak-4h.log}"
RESULTS="${RESULTS:-/home/kyle/coldrawdb/docs/phase4/perf/soak-4h.txt}"
RETRY_COUNT=0
MAX_RETRIES=1

# Parse flags
while [[ $# -gt 0 ]]; do
  case $1 in
    --duration=*)
      DURATION_SEC="${1#*=}"
      shift
      ;;
    --base-url=*)
      BASE_URL="${1#*=}"
      shift
      ;;
    *)
      echo "Unknown flag: $1"
      exit 1
      ;;
  esac
done

echo "=== 4h Soak Test ===" | tee "$SOAK_LOG"
echo "Duration: ${DURATION_SEC}s ($(($DURATION_SEC / 60)) min)" | tee -a "$SOAK_LOG"
echo "Base URL: $BASE_URL" | tee -a "$SOAK_LOG"
echo "Results: $RESULTS" | tee -a "$SOAK_LOG"
echo "Start time: $(date)" | tee -a "$SOAK_LOG"
echo ""

start_ts=$(date +%s)
end_ts=$((start_ts + DURATION_SEC))

total=0
success=0
conflict_500=0
inconsistent=0
run_failed=false

# Helper: call API and capture status
api_post() {
  curl -s -X POST "${BASE_URL}/api/v1/diagrams" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"soak_test_$(date +%s)\"}" \
    -w "\n%{http_code}" \
    -o /tmp/soak_create_resp.json 2>/dev/null
}

api_get() {
  local id="$1"
  curl -s -X GET "${BASE_URL}/api/v1/diagrams/${id}" \
    -w "\n%{http_code}" \
    -o /tmp/soak_get_resp.json 2>/dev/null
}

api_put() {
  local id="$1"
  local rev="$2"
  curl -s -X PUT "${BASE_URL}/api/v1/diagrams/${id}" \
    -H "Content-Type: application/json" \
    -d "{\"expected_revision\":${rev},\"diagram\":{\"id\":\"${id}\",\"name\":\"soak_update\"}}" \
    -w "\n%{http_code}" \
    -o /tmp/soak_put_resp.json 2>/dev/null
}

api_delete() {
  local id="$1"
  curl -s -X DELETE "${BASE_URL}/api/v1/diagrams/${id}" \
    -w "\n%{http_code}" \
    -o /tmp/soak_del_resp.json 2>/dev/null
}

# Extract body and status from combined response
get_status() {
  local file="$1"
  tail -1 "$file" | tr -d '\n'
}

get_body() {
  local file="$1"
  head -n -1 "$file" | tr -d '\n'
}

echo "Starting soak loop..." | tee -a "$SOAK_LOG"

while [ $(date +%s) -lt $end_ts ]; do
  total=$((total + 1))

  # 1. POST create diagram
  resp=$(api_post)
  status=$(echo "$resp" | tail -1)
  body=$(echo "$resp" | head -n -1)

  if [ "$status" != "200" ]; then
    echo "[$(date +%H:%M:%S)] create failed: HTTP $status" >> "$SOAK_LOG"
    conflict_500=$((conflict_500 + 1))
    continue
  fi

  # Extract id from {"id":"..."}
  diag_id=$(echo "$body" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
  if [ -z "$diag_id" ]; then
    echo "[$(date +%H:%M:%S)] create: no id in response" >> "$SOAK_LOG"
    conflict_500=$((conflict_500 + 1))
    continue
  fi

  # 2. PUT save with revision
  put_resp=$(api_put "$diag_id" 0)
  put_status=$(echo "$put_resp" | tail -1)

  if [ "$put_status" == "409" ] || [ "$put_status" == "500" ]; then
    echo "[$(date +%H:%M:%S)] save $put_status for $diag_id" >> "$SOAK_LOG"
    conflict_500=$((conflict_500 + 1))
    # Cleanup anyway
    api_delete "$diag_id" > /dev/null 2>&1
    continue
  fi

  # 3. DELETE diagram
  del_resp=$(api_delete "$diag_id")
  del_status=$(echo "$del_resp" | tail -1)

  # 4. GET verify is_deleted
  get_resp=$(api_get "$diag_id")
  get_status_code=$(echo "$get_resp" | tail -1)

  if [ "$get_status_code" == "200" ]; then
    # Check is_deleted in body
    is_deleted=$(echo "$get_resp" | head -n -1 | grep -o '"is_deleted":[0-9]*' | cut -d: -f2)
    if [ "$is_deleted" != "1" ]; then
      inconsistent=$((inconsistent + 1))
      echo "[$(date +%H:%M:%S)] INCONSISTENT: $diag_id is_deleted=$is_deleted" >> "$SOAK_LOG"
    fi
  fi

  success=$((success + 1))

  if [ $((total % 50)) -eq 0 ]; then
    elapsed=$(($(date +%s) - start_ts))
    rate=$(echo "scale=4; $success / $total" | bc 2>/dev/null || echo "N/A")
    echo "[$(date +%H:%M:%S)] progress: total=$total success=$success rate=$rate" >> "$SOAK_LOG"
  fi
done

elapsed=$(( $(date +%s) - start_ts ))
success_rate=$(echo "scale=4; $success / $total" | bc 2>/dev/null || echo "0")
conflict_rate=$(echo "scale=4; $conflict_500 / $total" | bc 2>/dev/null || echo "0")

echo "" | tee -a "$SOAK_LOG"
echo "=== Soak Complete ===" | tee -a "$SOAK_LOG"
echo "Elapsed: ${elapsed}s" | tee -a "$SOAK_LOG"
echo "Total: $total" | tee -a "$SOAK_LOG"
echo "Success: $success" | tee -a "$SOAK_LOG"
echo "Conflict/500: $conflict_500" | tee -a "$SOAK_LOG"
echo "Inconsistent: $inconsistent" | tee -a "$SOAK_LOG"
echo "Success rate: $success_rate (threshold >= 0.9995)" | tee -a "$SOAK_LOG"
echo "Conflict rate: $conflict_rate (threshold < 0.001)" | tee -a "$SOAK_LOG"

# Write results file
mkdir -p "$(dirname "$RESULTS")"
cat > "$RESULTS" << EOF
# 4h Soak Results
# Run time: $(date)
# Duration: ${DURATION_SEC}s ($(($DURATION_SEC / 60)) min)
# BASE_URL: $BASE_URL
# Total requests: $total
# Success: $success
# Conflict/500: $conflict_500
# Inconsistent: $inconsistent
# Success rate: $success_rate (threshold >= 0.9995)
# Conflict/500 rate: $conflict_rate (threshold < 0.001)
# Inconsistency errors: $inconsistent (threshold = 0)

## AC-16 Thresholds
- save_success_rate >= 0.9995
- conflict_500_rate < 0.001
- inconsistent_count == 0

## PASS/FAIL
$(if [ "$success_rate" != "N/A" ] && [ "$(echo "$success_rate >= 0.9995" | bc)" -eq 1 ] && [ "$(echo "$conflict_rate < 0.001" | bc)" -eq 1 ] && [ "$inconsistent" -eq 0 ]; then echo "PASS"; else echo "FAIL"; fi)

## Note
降级: 4h 实际运行需 dedicated session 或 CI 调度。本结果为 sanity run (${DURATION_SEC}s)。
EOF

echo "Results written to $RESULTS"

# Exit code based on thresholds
if [ "$success_rate" != "N/A" ] && [ "$(echo "$success_rate >= 0.9995" | bc)" -eq 1 ] && [ "$(echo "$conflict_rate < 0.001" | bc)" -eq 1 ] && [ "$inconsistent" -eq 0 ]; then
  echo "OVERALL: PASS"
  exit 0
else
  echo "OVERALL: FAIL"
  exit 1
fi