# APS Runtime Provisioning & Project Config

| ID | Owner | Status |
|----|-------|--------|
| APSCFG | @joshuaboys | Ready |

## Purpose
Introduce a global-runtime / per-project provisioning model so APS behavior is configured by project policy instead of hardcoded defaults.

## In Scope

- Per-project APS config file (`.apsrc.yaml`)
- `aps init` preference prompts with persisted toggles
- Backward-compatible defaults when config is missing
- Config-aware lint/runtime hooks in subsequent slices

## Out of Scope *(optional)*

- Full remote profile registry
- Multi-file config inheritance
- Breaking schema changes without migration support

## Interfaces *(optional)*

**Depends on:**

- APS CLI scaffold/update flow
- Lint/runtime rule loading

**Exposes:**

- Stable config schema (`version: 1`)
- Project-level toggles for planning/execution/verification gates

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Tasks

### APSCFG-001: Scaffold `.apsrc.yaml` during `aps init`
- **Intent:** Persist project APS policy at bootstrap instead of relying on implicit defaults.
- **Expected Outcome:** `aps init` writes `.apsrc.yaml` with default keys and values.
- **Validation:** `./bin/aps init /tmp/aps-init-test && test -f /tmp/aps-init-test/.apsrc.yaml`

### APSCFG-002: Add init-time preference capture
- **Intent:** Let project owners select key behavior toggles during init.
- **Expected Outcome:** Interactive `aps init` asks a minimal set of config questions and persists answers.
- **Validation:** `script -q /tmp/aps-init-capture.log ./bin/aps init /tmp/aps-init-interactive`

### APSCFG-003: Config-aware runtime loading (non-breaking)
- **Intent:** Ensure runtime can read `.apsrc.yaml` and falls back safely when absent.
- **Expected Outcome:** CLI loads config when present and behaves identically to current defaults when not.
- **Validation:** `./bin/aps lint . && rm -f .apsrc.yaml && ./bin/aps lint .`

### APSCFG-004: Migrations & diagnostics
- **Intent:** Make schema changes safe and observable.
- **Expected Outcome:** Add migration/doctor workflow contract for future schema bumps.
- **Validation:** `./bin/aps --help` includes migration/doctor path (or documented placeholder) and docs reference migration flow.

### APSCFG-005: Future option — lint gateway enforcement (deferred)
- **Intent:** Keep strict planning-rule enforcement as an explicit future option without blocking current rollout.
- **Expected Outcome:** Planning docs record lint-gateway as deferred (`Maybe/Future`) with trigger conditions for adoption.
- **Validation:** `plans/index.aps.md` or module notes include deferred-item reference.

## Execution *(optional)*

Steps: [../execution/APSCFG.steps.md](../execution/APSCFG.steps.md)
