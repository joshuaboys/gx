# Contributing to gx

## Prerequisites

- [Bun](https://bun.sh) v1.0+
- Git

## Setup

```sh
git clone https://github.com/joshuaboys/gx
cd gx
bun install
```

## Development

Run from source (no build step):

```sh
bun run dev
```

Build the standalone binary:

```sh
bun run build
```

Run tests:

```sh
bun test
```

## Project structure

```
src/
  index.ts              # CLI entry point and command router
  commands/
    clone.ts            # gx clone
    config.ts           # gx config
    init.ts             # gx init (.claude/ scaffolding)
    ls.ts               # gx ls
    open.ts             # gx open
    rebuild.ts          # gx rebuild
    resolve.ts          # gx <name> (project lookup)
    shell-init.ts       # gx shell-init (shell integration generator)
  lib/
    config.ts           # Config file management (~/.config/gx/config.json)
    detect.ts           # Project type detection
    fuzzy.ts            # Fuzzy matching for project names
    index.ts            # Lib barrel exports
    path.ts             # Path resolution and project indexing
    templates.ts        # .claude/ scaffolding templates
    url.ts              # Git URL parsing
plugin/
  gx.plugin.zsh        # oh-my-zsh compatibility shim
install.sh              # Curl installer
```

## Conventions

- TypeScript, built with Bun
- No external runtime dependencies (compiles to standalone binary)
- Shell integration generated at runtime via `gx shell-init`
- Conventional commits (`feat:`, `fix:`, `docs:`, etc.)
