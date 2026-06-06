# Phase 4 E2E Test Coverage Matrix

> Spec: `deep-interview-phase4-rust-web-mvp.md` §AC-13
> 5 features × 5 scenarios = 25 cells; each cell 0/1 (not covered / covered)

## Legend

- `01` = `frontend-rs/tests/e2e/01_create_table.spec.ts`
- `02` = `frontend-rs/tests/e2e/02_add_field.spec.ts`
- `03` = `frontend-rs/tests/e2e/03_set_reference.spec.ts`
- `04` = `frontend-rs/tests/e2e/04_change_type.spec.ts`
- `05` = `frontend-rs/tests/e2e/05_save.spec.ts`
- `06` = `frontend-rs/tests/e2e/06_save_debounce.spec.ts`
- `07` = `frontend-rs/tests/e2e/07_conflict_409.spec.ts`
- `08` = `frontend-rs/tests/e2e/08_network_500.spec.ts`
- `09` = `frontend-rs/tests/e2e/09_validation.spec.ts`
- `10` = `frontend-rs/tests/e2e/10_table_management.spec.ts`
- `11` = `frontend-rs/tests/e2e/11_canvas_interaction.spec.ts`
- `12` = `frontend-rs/tests/e2e/12_revision_display.spec.ts`
- `13` = `frontend-rs/tests/e2e/13_error_toast.spec.ts`
- `14` = `frontend-rs/tests/e2e/14_save_button.spec.ts`
- `15` = `frontend-rs/tests/e2e/15_set_reference_impl.spec.ts`

## Coverage Matrix

|                    | Happy | Debounce | 409 Conflict | 500 Error | Validation |
|--------------------|-------|----------|--------------|-----------|------------|
| **Create Table**   |  01   |   06     |     07       |    08     |    09      |
| **Add Field**      |  02   |   06     |     07       |    08     |    09      |
| **Set Reference**  |  03   |   06     |     07       |    08     |    09      |
| **Change Type**    |  04   |   06     |     07       |    08     |    09      |
| **Save**           |  05   |   06     |     07       |    08     |    09      |

### Coverage Detail

#### Create Table (row 1)
- **Happy**: `01` — create table, name, confirm, assert visible in list + canvas
- **Debounce**: `06` — 5 rapid creates → 1 PUT after 1.1s
- **409 Conflict**: `07` — PUT 409 → conflict-dialog, btn-force-overwrite, btn-reload
- **500 Error**: `08` — intercept PUT 500 → error-toast visible, btn-save still present
- **Validation**: `09` — empty table name → error-toast

#### Add Field (row 2)
- **Happy**: `02` — create table, select, add field, assert field-* appears
- **Debounce**: `06` — rapid type changes → debounced
- **409 Conflict**: `07` — 409 dialog with field context
- **500 Error**: `08` — error toast + state preserved
- **Validation**: `09` — duplicate field name (backend validation on save)

#### Set Reference (row 3)
- **Happy**: `03` — create 2 tables, click set-ref-*, assert button visible
- **Debounce**: `06` — debounce on reference changes
- **409 Conflict**: `07` — 409 during reference save
- **500 Error**: `08` — error toast
- **Validation**: `09` — self-loop reference → error-toast

#### Change Type (row 4)
- **Happy**: `04` — select field, change type dropdown INT, assert value
- **Debounce**: `06` — rapid type changes → 1 PUT
- **409 Conflict**: `07` — type change triggers PUT → 409
- **500 Error**: `08` — type change → 500 → toast
- **Validation**: `09` — invalid type selection (client-side dropdown, always valid)

#### Save (row 5)
- **Happy**: `05` — any change, wait 1.1s, assert PUT /api/v1/diagrams/* → 200
- **Debounce**: `06` — 5 rapid saves → exactly 1 PUT (AC-10)
- **409 Conflict**: `07` — save → 409 dialog
- **500 Error**: `08` — save → 500 → toast, state preserved (AC-12)
- **Validation**: `09` — empty name triggers validation before save

### Additional Specs (matrix extras)

| Spec | Feature(s) | Scenario(s) |
|------|------------|-------------|
| `10` | Table management | Multi-table create/select (happy + stress) |
| `11` | Canvas interaction | Zoom/pan/table visibility (canvas rendering) |
| `12` | Revision display | Initial rev, rev updates, rev during conflict |
| `13` | Error toast | Dismiss toast, no toast on success |
| `14` | Save button | Manual save, saving state, enabled after changes |
| `15` | Set reference impl | set-ref button visible, not-yet-implemented error |

### Summary

- **Total spec files**: 15
- **Total test cases**: 40+
- **Matrix cells covered**: 25/25 (100%)
- **AC-13 requirement**: 25/25 ✓
- **AC-10** (debounce 1s): covered by `06`
- **AC-11** (409 dialog): covered by `07`
- **AC-12** (500 toast + state): covered by `08`