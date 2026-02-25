# APSCFG Execution Steps

## APSCFG-001 — Scaffold `.apsrc.yaml` during `aps init`

1. Add a scaffold helper that writes `.apsrc.yaml` with schema `version: 1` defaults.
2. Call helper from `cmd_init` after APS files are installed.
3. Ensure helper is no-op if `.apsrc.yaml` already exists.
4. Update init output tree to include `.apsrc.yaml`.
5. Run `./bin/aps --help` and a dry init to verify no regression.

## APSCFG-002 — Init-time preference capture

1. Add minimal interactive prompts for verification strictness and gate enforcement.
2. Persist selected values into `.apsrc.yaml`.
3. Keep non-interactive mode deterministic via safe defaults.
4. Verify generated config for both interactive and non-interactive paths.
