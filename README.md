# gx

[![CI](https://github.com/joshuaboys/gx/actions/workflows/ci.yml/badge.svg)](https://github.com/joshuaboys/gx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Bun](https://img.shields.io/badge/Built%20with-Bun-f9f1e1?logo=bun)](https://bun.sh)

A fast git project manager for cloning, navigating, and organising repositories.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/joshuaboys/gx/main/install.sh | sh
```

This downloads (or builds) the `gx` binary, puts it on your PATH, and sets up shell integration with tab completion. Supports zsh, bash, and fish.

> **Note:** Shell integration is required for `gx` to `cd` into projects. The curl installer sets this up automatically. If you installed manually, add `eval "$(gx shell-init)"` to your shell config (see below).

<details>
<summary>Manual install</summary>

Requires [Bun](https://bun.sh) v1.0+.

```sh
git clone https://github.com/joshuaboys/gx
cd gx
bun install && bun run build
cp gx ~/.local/bin/
```

Add shell integration to your config file:

**zsh** (`~/.zshrc`) / **bash** (`~/.bashrc`):
```sh
eval "$(gx shell-init)"
```

**fish** (`~/.config/fish/conf.d/gx.fish`):
```fish
gx shell-init | source
```
</details>

<details>
<summary>oh-my-zsh (legacy)</summary>

If you already use gx as an oh-my-zsh plugin, it still works:

```sh
ln -s /path/to/gx/plugin ~/.oh-my-zsh/custom/plugins/gx
# add gx to plugins=(...) in ~/.zshrc
```

The plugin file now delegates to `gx shell-init` internally.
</details>

## Usage

### Clone a repository

```sh
gx clone user/repo              # GitHub shorthand
gx clone https://github.com/user/repo
gx clone git@github.com:user/repo.git
```

Repositories are cloned to `~/Projects/src/<owner>/<repo>` by default and the shell cd's into the new directory.

### Jump to a project

```sh
gx myproject        # exact match
gx myproj           # fuzzy match fallback
```

Tab completion works for all indexed project names.

### List projects

```sh
gx ls
```

### Open in editor

```sh
gx open myproject               # uses default editor
gx open myproject --editor code # override editor
gx open                         # open current directory
```

Editor resolution order: `--editor` flag > `gx config editor` > `$VISUAL` > `$EDITOR` > nano

### Scaffold AI agent config

```sh
gx init                    # auto-detect project type
gx init --type rust        # override detection
gx init --force            # overwrite existing .claude/
```

Creates `.claude/CLAUDE.md` and `.claude/commands/` with plan and review slash commands. Supports project types: `typescript-bun`, `typescript-node`, `rust`, `go`, `python`, `generic`.

## Configuration

```sh
gx config                         # show current config
gx config set projectDir ~/code   # change project directory
gx config set structure flat      # use repo-only layout
gx config set structure owner    # use owner/repo layout (default)
gx config set structure host     # use host/owner/repo layout
gx config set editor code         # set default editor
```

Config is stored at `~/.config/gx/config.json`.

### Directory structure

By default, gx uses the `owner` structure:

```
~/Projects/src/
  owner/repo/
  owner/other-repo/
```

Set `structure` to `flat` for a repo-only layout (no owner prefix):

```
~/Projects/src/
  repo/
  other-repo/
```

Set `structure` to `host` for host-prefixed layout:

```
~/Projects/src/
  github.com/owner/repo/
  gitlab.com/owner/other-repo/
```

### Rebuild index

```sh
gx rebuild
```

Rescans the project directory and rebuilds the project index.

## Uninstall

```sh
rm ~/.local/bin/gx
```

Remove the `# gx` block from your shell config file (`~/.zshrc`, `~/.bashrc`, or `~/.config/fish/conf.d/gx.fish`).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Acknowledgements

gx draws inspiration from these projects:

- **[ghq](https://github.com/x-motemen/ghq)** — the original structured repository manager. ghq pioneered the `host/owner/repo` directory layout and index-based project lookup that gx builds on.
- **[gclone](https://github.com/allonsy/gclone)** — a git clone helper with automatic directory organisation, shorthand URL parsing, and shell auto-cd. gx's clone workflow and shell integration owe a lot to gclone's approach.

## License

MIT
