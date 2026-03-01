# gx Use Cases & Examples

Real-world workflows where gx streamlines day-to-day development.

---

## 1. Rapid Context Switching

**Problem:** You're working on a backend API, get a Slack message about a frontend bug, and need to jump between repos fast.

```sh
gx api-server          # jump to the backend
# ... investigate, then switch:
gx dashboard           # jump to the frontend
# ... fix the bug, then return:
gx api-server           # back in seconds
```

Shell integration (`eval "$(gx shell-init)"`) is required for `gx <name>` to auto-`cd`. Without it, the command prints the resolved path to stdout instead of changing directories.

No need to remember directory paths or keep multiple terminals open per project.

---

## 2. Scripting with the Index

**Problem:** You want to run a command across all your indexed projects — check for uncommitted work, update dependencies, or gather stats.

```sh
# Find repos with uncommitted changes
gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  if [ -n "$(git -C "$dir" status --porcelain)" ]; then
    echo "dirty: $dir"
  fi
done
```

```sh
# Count lines of code across all projects
gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  count=$(find "$dir" -type f \( -name '*.ts' -o -name '*.go' -o -name '*.rs' \) -print0 | xargs -0 wc -l 2>/dev/null | tail -1)
  echo "$name: $count"
done
```

The index is a plain JSON file at `~/.config/gx/index.json`, so you can also query it directly with `jq`:

```sh
# List all project paths
jq -r '.projects | to_entries[] | .value.path' ~/.config/gx/index.json

# Find projects cloned from a specific org
jq -r '.projects | to_entries[] | select(.value.url | test("myorg")) | .key' ~/.config/gx/index.json
```

---

## 3. Onboarding onto a New Machine

**Problem:** You've got a fresh laptop and need to clone 20+ repos into a consistent layout.

```sh
# Install gx
curl -fsSL https://raw.githubusercontent.com/joshuaboys/gx/main/install.sh | sh

# Clone everything — each lands in ~/Projects/src/owner/repo
gx clone company/api-server
gx clone company/dashboard
gx clone company/shared-lib
gx clone company/infra
gx clone personal/dotfiles
# ...

# Verify
gx ls
```

Every repo follows the same directory convention. No more `~/code` vs `~/dev` vs `~/projects` chaos.

---

## 4. Polyglot Multi-Repo Development

**Problem:** Your system spans Go microservices, a TypeScript frontend, a Python ML pipeline, and Rust CLI tools. You need to move between them fluidly.

```sh
gx ml-pipeline          # Python project
gx auth-service          # Go microservice
gx web-app               # TypeScript frontend
gx cli-tools             # Rust binary
```

Fuzzy matching means you don't need exact names:

```sh
gx auth                  # matches auth-service
gx ml                    # matches ml-pipeline
```

---

## 5. Scaffolding AI Agent Configs Across Projects

**Problem:** You want every project to have a consistent `.claude/CLAUDE.md` with the right build/test/lint commands for its language.

```sh
# Auto-detect and scaffold each project
for name in $(gx resolve --list); do
  dir=$(gx resolve "$name")
  (cd "$dir" && gx init 2>/dev/null && echo "scaffolded: $name")
done
```

Each project gets a tailored `CLAUDE.md` — Bun commands for TypeScript projects, `cargo` for Rust, `go` for Go, etc.

---

## 6. Quick Code Review Across Repos

**Problem:** You review PRs across multiple repos and want to pull branches and open them in your editor without hunting for directories.

```sh
gx open api-server --editor code     # opens in VS Code
# review the PR, then move on:
gx open dashboard --editor code
```

Or open in the terminal:

```sh
gx open dotfiles --editor nvim
```

---

## 7. CI/CD & Automation Scripts

**Problem:** Your deploy script needs to reference paths for multiple repos — building artifacts, copying configs, running integration tests.

```sh
#!/bin/bash
# deploy.sh — uses gx resolve for consistent path lookup

API_DIR=$(gx resolve api-server)
WEB_DIR=$(gx resolve dashboard)
INFRA_DIR=$(gx resolve infra)

# Build
(cd "$API_DIR" && make build)
(cd "$WEB_DIR" && npm run build)

# Deploy
cp "$INFRA_DIR/k8s/prod.yaml" /tmp/deploy.yaml
# ...
```

`gx resolve` gives you a stable, scriptable way to get project paths without hardcoding.

---

## 8. Workspace Auditing

**Problem:** Over time you accumulate dozens of cloned repos. Which ones are stale? Which have work in progress?

```sh
# Projects with uncommitted changes
gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  dirty=$(git -C "$dir" status --porcelain 2>/dev/null | wc -l)
  if [ "$dirty" -gt 0 ]; then
    echo "$name: $dirty uncommitted files"
  fi
done
```

```sh
# Projects with unpushed commits
gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  ahead=$(git -C "$dir" rev-list --count @{u}..HEAD 2>/dev/null)
  if [ "$ahead" -gt 0 ] 2>/dev/null; then
    echo "$name: $ahead unpushed commits"
  fi
done
```

```sh
# Projects not touched in 30+ days
gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  last=$(git -C "$dir" log -1 --format=%ct 2>/dev/null)
  if [ -n "$last" ]; then
    age=$(( ($(date +%s) - last) / 86400 ))
    [ "$age" -gt 30 ] && echo "$name: ${age}d since last commit"
  fi
done
```

---

## 9. Feeding Context to LLMs & AI Tools

**Problem:** You want to provide an AI assistant with information about your project landscape — what repos you have, their languages, their locations.

```sh
# Generate a project manifest for an AI prompt
echo "My projects:" && gx resolve --list | while read -r name; do
  dir=$(gx resolve "$name")
  lang=""
  [ -f "$dir/package.json" ] && lang="TypeScript/JS"
  [ -f "$dir/Cargo.toml" ] && lang="Rust"
  [ -f "$dir/go.mod" ] && lang="Go"
  [ -f "$dir/pyproject.toml" ] && lang="Python"
  echo "  - $name ($lang) @ $dir"
done
```

Or use the index directly:

```sh
# Pipe the raw index to an LLM for analysis
cat ~/.config/gx/index.json | jq '.'
```

---

## 10. Team Standardisation

**Problem:** Your team has inconsistent repo layouts — some use `~/code`, others `~/dev/work`, paths break in shared scripts.

**Solution:** Everyone installs gx with the same config:

```sh
gx config set projectDir ~/Projects/src
gx config set structure owner
gx config set defaultHost github.com
```

Now `gx resolve api-server` returns the same relative layout on every machine. Shared scripts, documentation, and onboarding guides can reference `gx resolve <name>` instead of hardcoded paths.

---

## 11. Shell Aliases & Integrations

Build your own shortcuts on top of gx:

```sh
# Jump and open in one command
gxo() { gx "$1" && gx open --editor "${2:-code}"; }

# Jump and show git status
gxs() { gx "$1" && git status; }

# Jump and pull latest
gxp() { gx "$1" && git pull; }

# Clone and scaffold AI config
gxc() { gx clone "$1" && gx init; }
```

---

## 12. Monorepo-Adjacent Workflows

**Problem:** You work across related repos that aren't in a monorepo (e.g., a shared library, an API that consumes it, and a frontend). You need to coordinate changes.

```sh
# Check all three repos for dirty state
for repo in shared-lib api-server dashboard; do
  dir=$(gx resolve "$repo")
  branch=$(git -C "$dir" branch --show-current)
  dirty=$(git -C "$dir" status --porcelain | wc -l)
  echo "$repo ($branch): $dirty files changed"
done
```

```sh
# Open all related repos in VS Code workspaces
code "$(gx resolve shared-lib)" "$(gx resolve api-server)" "$(gx resolve dashboard)"
```

---

## Summary

| Use Case | Key Commands |
|----------|-------------|
| Context switching | `gx <name>` |
| Scripting with index | `gx resolve`, `gx resolve --list` |
| Machine onboarding | `gx clone`, `gx ls` |
| Multi-language navigation | `gx <name>` with fuzzy matching |
| AI agent scaffolding | `gx init` |
| Code review | `gx open --editor` |
| CI/CD path lookup | `gx resolve <name>` |
| Workspace auditing | `gx resolve --list` + git commands |
| LLM context generation | Index JSON + `gx resolve --list` |
| Team standardisation | `gx config set` |
| Custom shell aliases | Shell functions wrapping gx |
| Multi-repo coordination | `gx resolve` in loops |
