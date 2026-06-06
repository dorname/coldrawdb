# Phase 4 Rollback Gate

> Spec: `deep-interview-phase4-rust-web-mvp.md` §R-5
> Defines three pause conditions that halt the React removal workflow.

## Three Pause Conditions

### P1: Main Branch CI Red

**Trigger**: `.github/workflows/build.yml` (post W4-3 CI改造) exits non-zero on `main` or `feature/phase4-react-removal`.

**Detection**:
```bash
git checkout main && git pull origin main
./.github/workflows/build.yml   # non-zero exit
```

**Decision Tree**:
```
CI fails on main
├── Identify failing step (cargo build / cargo test / trunk build / wasm-pack test)
├── If wasm-pack test --chrome fails → rollback W4-3 CI changes (do not merge)
├── If backend test退化 → fix backend test first, re-run CI
├── If frontend-rs build fails → editor-render / panels not ready, wait for W2 workers
└── If CI still fails after 2 attempts → escalate to team-lead → STOP merge to main
```

**Rollback**: `git revert <last-merge-commit>` on feature branch; do not push to main.

---

### P2: 4h Soak Failure

**Trigger**: `frontend-rs/tests/soak/4h.sh` reports:
- `save_success_rate < 0.9995` (threshold AC-16)
- OR `conflict_500_rate >= 0.001` (threshold AC-16)
- OR `inconsistent_count > 0` (threshold AC-16)

**Detection**:
```bash
cd /home/kyle/coldrawdb/frontend-rs
bash tests/soak/4h.sh
# output → docs/phase4/perf/soak-4h.txt
grep "save_success_rate" docs/phase4/perf/soak-4h.txt
# compare against thresholds
```

**Decision Tree**:
```
4h soak reports failure
├── First failure: auto-retry 1 time (built into 4h.sh)
│   └── If retry succeeds → resume
├── Continuous 2nd failure:
│   ├── DO NOT merge feature/phase4-react-removal to main
│   ├── Preserve feature branch for debugging
│   ├── Notify team-lead immediately
│   └── Escalate: rollback decision (spec amendment or wait for fix)
└── If consistent failure after rollback → W4-3 PAUSE LINE triggered
```

**Recovery**: Run `docs/phase4/DB_BACKUP_BEFORE_W4.md` restoration steps if rollback executed.

---

### P3: E2E Coverage < 25/25

**Trigger**: `find frontend-rs/tests/e2e -name '*.spec.*' | wc -l` returns < 15 (minimum spec count) OR `test-coverage-matrix.md` shows < 25 cells covered.

**Detection**:
```bash
cd /home/kyle/coldrawdb/frontend-rs
find tests/e2e -name '*.spec.*' | wc -l   # must be >= 15
# AND
cat docs/phase4/test-coverage-matrix.md | grep -c " 1 "   # must be 25
```

**Decision Tree**:
```
E2E count < 15 OR matrix < 25/25
├── If happy path missing → write missing specs immediately
├── If exception path missing → write missing exception specs
├── If CI not enforcing → add coverage gate to .github/workflows/build.yml
│   └── Required: E2E spec count >= 15 AND matrix 25/25
└── If cannot reach 25/25 in current session → escalate to team-lead → PAUSE
```

---

## Combined Decision Flow

```
                    START: Phase 4 W4-1
                            |
            ┌───────────────┼───────────────┐
            v               v               v
         CI Green      4h Soak Pass    E2E >= 25/25
            |               |               |
            v               v               v
      Continue to      Continue to      Continue to
      W4-2 CI改造      W4-2 CI改造      W4-2 CI改造
            \               |               /
             \              |              /
              v             v             v
                    ALL GREEN
                        |
                        v
               Merge feature/phase4-react-removal
               to main (W4-2 Step 4)
                        |
              ┌─────────┼─────────┐
              v         v         v
           CI Red   Soak Fail  E2E < 25
              |         |         |
              v         v         v
           REVERT   REVERT    WRITE MISSING
         (W4-3)   (W4-3)    (retry once)
              \         |         /
               v        v        v
                 PAUSE + ESCALATE
               (team-lead decision)
```

## Rollback Procedure

### Step 0: DB Backup (already done in W4-2 Step 0)

If rollback is triggered, database is already backed up:
```bash
cp backend/db.sqlite backups/phase4-pre-rollback-$(date +%Y%m%d-%H%M).sqlite
```

### Step 1: Revert Feature Branch

```bash
git checkout feature/phase4-react-removal
git revert <merge-commit-sha>
git push origin feature/phase4-react-removal
```

### Step 2: Restore React (if needed)

```bash
git checkout main
git tag -d phase4-pre-react-removal 2>/dev/null || true
# React files are preserved via git history; no special action needed
```

### Step 3: Notify

- Notify team-lead immediately on any pause condition
- Document incident in `docs/phase4/INCIDENT_YYYYMMDD.md`
- Schedule post-mortem before resuming

---

## Sign-off

| Role | Name | Date |
|------|------|------|
| Architect | | |
| Team Lead | | |
| Product Owner | | |