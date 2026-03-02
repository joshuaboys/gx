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

See [AGENTS.md](AGENTS.md) for the full project structure and file descriptions.

## Conventions

- TypeScript, built with Bun
- No external runtime dependencies (compiles to standalone binary)
- Shell integration generated at runtime via `gx shell-init`
- Conventional commits (`feat:`, `fix:`, `docs:`, etc.)
