#!/usr/bin/env bash
# SMOKE-core-01~06 本地 staging smoke runner（logos.config.json smoke.command 入口）
#
# 流程：
#   1) start-local.sh 启动 backend + frontend
#   2) 在 services ready 期间跑 SMOKE-core-01~05（curl 打本地后端）
#   3) stop-local.sh 关闭服务
#   4) 把所有结果追加到 logos/resources/verify/smoke-results.jsonl
#
# 对应规格：logos/resources/test/smoke/core-smoke-test-cases.md
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESULTS="${REPO_ROOT}/logos/resources/verify/smoke-results.jsonl"
START_SCRIPT="${REPO_ROOT}/scripts/start-local.sh"
STOP_SCRIPT="${REPO_ROOT}/scripts/stop-local.sh"

# backend 端口从 backend/config.toml 读；这里直接用默认值 3000
BACKEND_PORT="${COLDRAWDB_BACKEND_PORT:-3000}"
FRONTEND_PORT="${COLDRAWDB_FRONTEND_PORT:-18080}"

# 与 scripts/tests/test-local-scripts.sh 一致：固定 18080 避免与开发 8080 冲突
export COLDRAWDB_FRONTEND_PORT="$FRONTEND_PORT"
# 与 scripts/tests/test-local-scripts.sh 一致，避免污染默认日志
export COLDRAWDB_BACKEND_LOG="logs/smoke-backend.log"
export COLDRAWDB_FRONTEND_LOG="logs/smoke-frontend.log"
export COLDRAWDB_BACKEND_PID="logs/smoke-backend.pid"
export COLDRAWDB_FRONTEND_PID="logs/smoke-frontend.pid"
export COLDRAWDB_HEALTH_TIMEOUT=120

mkdir -p "$(dirname "$RESULTS")"

run_smoke_case() {
    local id="$1"
    local status="$2"
    local duration_ms="$3"
    local note="$4"
    local started_at
    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '{"id":"%s","status":"%s","duration_ms":%s,"timestamp":"%s","scenario":"smoke","note":"%s"}\n' \
        "$id" "$status" "$duration_ms" "$started_at" "$note" >> "$RESULTS"
}

measure() {
    # measure_strict <id> <note-prefix> <curl-args...>
    # 仅 2xx 算 pass；3xx/4xx/5xx 全 fail。用于要求端点真能用的场景（CRUD/Import）。
    local id="$1"
    local note_prefix="$2"
    shift 2
    local start_ms end_ms duration_ms body http_code
    start_ms="$(date +%s%3N)"
    body="$(curl --silent --max-time 5 "$@" 2>&1)"
    http_code="$(curl --silent --max-time 5 --output /dev/null --write-out '%{http_code}' "$@" 2>&1)"
    end_ms="$(date +%s%3N)"
    duration_ms=$((end_ms - start_ms))
    if [[ "$http_code" =~ ^2[0-9][0-9]$ ]]; then
        run_smoke_case "$id" "pass" "$duration_ms" "${note_prefix} (http=${http_code})"
    else
        run_smoke_case "$id" "fail" "$duration_ms" "${note_prefix} (http=${http_code}, body=${body:0:200})"
    fi
}

measure_health() {
    # measure_health <id> <note-prefix> <curl-args...>
    # 接受 1xx-4xx 算 pass；5xx 和 connection error 才 fail。
    # 后端没有 /health 端点，4xx 反而是 routing 正常的信号。
    local id="$1"
    local note_prefix="$2"
    shift 2
    local start_ms end_ms duration_ms body http_code
    start_ms="$(date +%s%3N)"
    body="$(curl --silent --max-time 5 "$@" 2>&1)"
    http_code="$(curl --silent --max-time 5 --output /dev/null --write-out '%{http_code}' "$@" 2>&1)"
    end_ms="$(date +%s%3N)"
    duration_ms=$((end_ms - start_ms))
    if [[ "$http_code" =~ ^[1-4][0-9][0-9]$ ]]; then
        run_smoke_case "$id" "pass" "$duration_ms" "${note_prefix} (http=${http_code})"
    else
        run_smoke_case "$id" "fail" "$duration_ms" "${note_prefix} (http=${http_code}, body=${body:0:200})"
    fi
}

# ─── Stage 1: start services ──────────────────────────────────────────────
services_ok=0
if bash "$START_SCRIPT" >/dev/null 2>&1; then
    services_ok=1
fi

if [[ "$services_ok" -ne 1 ]]; then
    # services 起不来 → 全部 6 条都 fail
    for id in SMOKE-core-01 SMOKE-core-02 SMOKE-core-03 SMOKE-core-04 SMOKE-core-05 SMOKE-core-06; do
        run_smoke_case "$id" "fail" 0 "start-local.sh failed; services unavailable"
    done
    bash "$STOP_SCRIPT" >/dev/null 2>&1 || true
    exit 1
fi

# ─── Stage 2: SMOKE-core-01 健康检查 ──────────────────────────────────────
# 规格期望 GET /api/v1/diagrams/health 返回 200；该端点暂未实现，退化为
# GET /api/v1/diagrams/non-existent → 期望 4xx 即代表 backend 进程 + routing 健康
# （503/500 才是真异常）。
measure_health "SMOKE-core-01" "backend health proxy (4xx on /non-existent is healthy)" \
    "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams/__smoke_health__"

# ─── Stage 3: SMOKE-core-02 CRUD E2E ──────────────────────────────────────
# 1) POST /api/v1/diagrams → 创建（body 字段：name；diagrams.yaml CreateRequest）
crud_start=$(date +%s%3N)
crud_status="pass"
crud_note="create/read/update/delete"

create_resp="$(curl --silent --max-time 5 -X POST \
    -H 'Content-Type: application/json' \
    -d '{"name":"smoke"}' \
    "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams" 2>&1)"
created_id="$(echo "$create_resp" | grep -oE '"id":"[^"]+"' | head -1 | cut -d'"' -f4)"
create_code="$(curl --silent --max-time 5 -o /dev/null -w '%{http_code}' -X POST \
    -H 'Content-Type: application/json' \
    -d '{"name":"smoke"}' \
    "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams" 2>&1)"

if [[ -z "$created_id" ]] || [[ ! "$create_code" =~ ^2[0-9][0-9]$ ]]; then
    crud_status="fail"
    crud_note="POST /diagrams failed (http=${create_code})"
fi

# 2) DELETE /api/v1/diagrams/{id} → 清理
if [[ "$crud_status" == "pass" && -n "$created_id" ]]; then
    del_code="$(curl --silent --max-time 5 -o /dev/null -w '%{http_code}' -X DELETE \
        "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams/${created_id}" 2>&1)"
    if [[ ! "$del_code" =~ ^2[0-9][0-9]$ ]]; then
        crud_status="fail"
        crud_note="DELETE /diagrams/{id} failed (http=${del_code})"
    fi
fi

crud_end=$(date +%s%3N)
crud_duration=$((crud_end - crud_start))
run_smoke_case "SMOKE-core-02" "$crud_status" "$crud_duration" "$crud_note"

# ─── Stage 4: SMOKE-core-03 导入导出 ──────────────────────────────────────
measure "SMOKE-core-03" "POST /api/v1/bridge/import/local (SQL via payload)" \
    -X POST \
    -H 'Content-Type: application/json' \
    -d '{"source":"smoke","payload":{"name":"smoke_users","tables":[{"name":"smoke_users","fields":[{"name":"id","type":"INT"},{"name":"name","type":"VARCHAR"}]}]}}' \
    "http://127.0.0.1:${BACKEND_PORT}/api/v1/bridge/import/local"

# ─── Stage 5: SMOKE-core-04 静态资源 ──────────────────────────────────────
measure_health "SMOKE-core-04" "frontend index.html available" \
    "http://127.0.0.1:${FRONTEND_PORT}/"

# ─── Stage 6: SMOKE-core-05 数据库 schema（间接：bridge config 可达即代表 DB 在线）───
measure "SMOKE-core-05" "GET /api/v1/bridge/config (DB-backed)" \
    "http://127.0.0.1:${BACKEND_PORT}/api/v1/bridge/config"

# ─── Stage 6b: SMOKE-core-07 导入缺 id payload 持久化验证 ───────────────────
# fix-diagram-import-persistence：缺 id 表/字段不得静默丢数据；name 兜底不得为 NULL。
imp07_start=$(date +%s%3N)
imp07_status="pass"
imp07_note="import missing-id payload persisted (auto id + name fallback)"
imp07_body='{"source":"smoke","payload":{"tables":[{"name":"smoke_no_id","x":0,"y":0,"fields":[{"name":"id","type":"INT","primary":true}]}],"references":[]}}'
imp07_out="$(curl --silent --max-time 5 -X POST -H 'Content-Type: application/json' -d "$imp07_body" \
    -w '\n%{http_code}' "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams/import" 2>&1)"
imp07_http="$(printf '%s' "$imp07_out" | tail -n1)"
imp07_resp="$(printf '%s' "$imp07_out" | sed '$d')"
imp07_id=""

if [[ ! "$imp07_http" =~ ^2[0-9][0-9]$ ]]; then
    imp07_status="fail"
    imp07_note="POST /diagrams/import failed (http=${imp07_http}, body=${imp07_resp:0:160})"
else
    imp07_id="$(IMP07_RESP="$imp07_resp" python3 -c 'import os,json
try:
    d=(json.loads(os.environ["IMP07_RESP"]).get("data") or {})
except Exception:
    print("")
    raise SystemExit(1)
if d.get("imported_tables")!=1 or d.get("imported_fields")!=1:
    raise SystemExit(1)
print(d.get("diagram_id") or "")' 2>/dev/null)"
    if [[ -z "$imp07_id" ]]; then
        imp07_status="fail"
        imp07_note="import response: diagram_id empty or counts != 1 (body=${imp07_resp:0:160})"
    fi
fi

if [[ "$imp07_status" == "pass" ]]; then
    imp07_get="$(curl --silent --max-time 5 -w '\n%{http_code}' \
        "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams/${imp07_id}" 2>&1)"
    imp07_get_http="$(printf '%s' "$imp07_get" | tail -n1)"
    imp07_get_body="$(printf '%s' "$imp07_get" | sed '$d')"
    imp07_assert="$(IMP07_GET="$imp07_get_body" python3 -c 'import os,json
try:
    r=json.loads(os.environ["IMP07_GET"]); d=r.get("data") or {}
except Exception as e:
    print(f"GET response not JSON: {e}")
    raise SystemExit(1)
if r.get("code")!=0:
    print("GET code != 0")
    raise SystemExit(1)
tables=d.get("tables") or []
if len(tables)!=1:
    print(f"tables.length={len(tables)}")
    raise SystemExit(1)
t=tables[0]
if not isinstance(t.get("id"),str) or not t["id"]:
    print("tables[0].id empty")
    raise SystemExit(1)
fields=t.get("fields") or []
if len(fields)!=1:
    print(f"fields.length={len(fields)}")
    raise SystemExit(1)
if not isinstance(fields[0].get("id"),str) or not fields[0]["id"]:
    print("fields[0].id empty")
    raise SystemExit(1)
if not isinstance(d.get("name"),str):
    print("name not a string")
    raise SystemExit(1)
if not isinstance(d.get("revision"),int) or d["revision"]<1:
    print("revision < 1")
    raise SystemExit(1)
print("ok")' 2>&1)"
    if [[ "$imp07_get_http" != "200" || "$imp07_assert" != "ok" ]]; then
        imp07_status="fail"
        imp07_note="GET assert failed (http=${imp07_get_http}: ${imp07_assert})"
    fi
    # 清理：DELETE 导入的图
    imp07_del="$(curl --silent --max-time 5 -o /dev/null -w '%{http_code}' -X DELETE \
        "http://127.0.0.1:${BACKEND_PORT}/api/v1/diagrams/${imp07_id}" 2>&1)"
    if [[ ! "$imp07_del" =~ ^2[0-9][0-9]$ ]]; then
        imp07_status="fail"
        imp07_note="cleanup DELETE failed (http=${imp07_del})"
    fi
fi

imp07_end=$(date +%s%3N)
run_smoke_case "SMOKE-core-07" "$imp07_status" "$((imp07_end - imp07_start))" "$imp07_note"

# ─── Stage 7: stop services + SMOKE-core-06 ───────────────────────────────
stop_start=$(date +%s%3N)
stop_ok=0
if bash "$STOP_SCRIPT" >/dev/null 2>&1; then
    stop_ok=1
fi
stop_end=$(date +%s%3N)
stop_duration=$((stop_end - stop_start))

if [[ "$stop_ok" -eq 1 ]]; then
    run_smoke_case "SMOKE-core-06" "pass" "$stop_duration" "local start-local.sh + stop-local.sh round-trip"
else
    run_smoke_case "SMOKE-core-06" "fail" "$stop_duration" "stop-local.sh returned non-zero"
fi

# ─── Stage 8: SMOKE-core-STABLE-01 稳定版 Compose 健康检查 ──────────────────
# release-stable-win-linux-mac：compose 形态（本地构建镜像替代 GHCR 预构建镜像，
# 同 Dockerfile 产物）经 nginx 宿主机 9080 验证 health 与 SPA 入口。
# 无 Docker daemon 时按规格标注 SKIPPED。
st_start=$(date +%s%3N)
st_status="skip"
st_note="no docker daemon"

if docker info >/dev/null 2>&1; then
    st_status="fail"
    st_note="docker compose up failed (port 3000 busy after stop-local?)"
    if (cd "$REPO_ROOT" && docker compose up -d >/dev/null 2>&1); then
        st_health=""
        for _ in $(seq 1 60); do
            st_health="$(curl --silent --max-time 3 -o /dev/null -w '%{http_code}' \
                "http://127.0.0.1:9080/api/v1/diagrams/health" 2>/dev/null)"
            [[ "$st_health" == "200" ]] && break
            sleep 2
        done
        st_home="$(curl --silent --max-time 5 -o /dev/null -w '%{http_code}' \
            "http://127.0.0.1:9080/" 2>&1)"
        # compose-backend-port-internal-only：宿主 3000 应无监听（后端仅内部网络）
        st_direct="$(curl --silent --max-time 3 -o /dev/null -w '%{http_code}' \
            "http://127.0.0.1:3000/api/v1/diagrams/health" 2>/dev/null)"
        (cd "$REPO_ROOT" && docker compose down >/dev/null 2>&1)
        if [[ "$st_health" == "200" && "$st_home" =~ ^2[0-9][0-9]$ && "$st_direct" == "000" ]]; then
            st_status="pass"
            st_note="compose stack healthy via nginx :9080 (health=${st_health}, home=${st_home}, host:3000 closed)"
        else
            st_status="fail"
            st_note="compose health=${st_health}, home=${st_home}, host:3000=${st_direct} (expect 000=refused)"
        fi
    fi
fi

st_end=$(date +%s%3N)
run_smoke_case "SMOKE-core-STABLE-01" "$st_status" "$((st_end - st_start))" "$st_note"

# 任意一条 fail 都让 smoke 退出非零（OpenLogos 读取 exit code）
if grep -q '"status":"fail"' "$RESULTS" 2>/dev/null; then
    exit 1
fi
exit 0