<div align="center">

# gx

**A fast git project manager. 
Clone, jump, and organise repos from the terminal.**

[![CI](https://github.com/joshuaboys/gx/actions/workflows/ci.yml/badge.svg)](https://github.com/joshuaboys/gx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Bun](https://img.shields.io/badge/Built%20with-Bun-f9f1e1?logo=bun)](https://bun.sh)

</div>

---

## Features

- **Instant project switching** — jump to any repo by name with fuzzy matching
- **Structured organisation** — repos are cloned into a consistent `owner/repo` layout
- **GitHub shorthand** — `gx clone user/repo` just works
- **Shell integration** — tab completion and auto-`cd` for zsh, bash, and fish
- **Open in any editor** — `gx open` launches VS Code, nvim, or whatever you use
- **AI agent scaffolding** — `gx init` generates `.claude/` configs tailored to your project's language
- **Single binary** — zero runtime dependencies, compiles with Bun

## Quick Start

```sh
# Install
curl -fsSL https://raw.githubusercontent.com/joshuaboys/gx/main/install.sh | sh
exec $SHELL   # reload to pick up PATH and shell integration

# Clone a repo and cd into it
gx clone user/repo

# Jump back to it later
gx myproject
```

That's it. Shell integration and tab completion are set up automatically.

## Install

The curl installer downloads (or builds) the `gx` binary, puts it on your `PATH`, and sets up shell integration with tab completion.

```sh
curl -fsSL https://raw.githubusercontent.com/joshuaboys/gx/main/install.sh | sh
```

> **Note:** Shell integration is required for `gx` to `cd` into projects. The installer sets this up automatically. If you installed manually, add `eval "$(gx shell-init)"` for bash/zsh or `gx shell-init | source` for fish to your shell config.

<details>
<summary><strong>Manual install</strong></summary>

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
<summary><strong>oh-my-zsh (legacy)</strong></summary>

If you already use gx as an oh-my-zsh plugin, it still works:

```sh
ln -s /path/to/gx/plugin ~/.oh-my-zsh/custom/plugins/gx
# add gx to plugins=(...) in ~/.zshrc
```

The plugin file now delegates to `gx shell-init` internally.

</details>

## Usage

### Jump to a project

```sh
gx myproject        # exact match
gx myproj           # fuzzy match fallback
```

Tab completion works for all indexed project names.

### Clone a repository

```sh
gx clone user/repo              # GitHub shorthand
gx clone https://github.com/user/repo
gx clone git@github.com:user/repo.git
```

Repositories are cloned to `~/Projects/src/<owner>/<repo>` by default and the shell `cd`s into the new directory.

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

### Rebuild index

```sh
gx rebuild
```

Rescans the project directory and rebuilds the project index.

## Configuration

```sh
gx config                         # show current config
gx config set projectDir ~/code   # change project directory
gx config set structure flat      # use repo-only layout
gx config set structure owner     # use owner/repo layout (default)
gx config set structure host      # use host/owner/repo layout
gx config set editor code         # set default editor
```

Config is stored at `~/.config/gx/config.json`.

### Directory structures

<table>
<tr><th><code>owner</code> (default)</th><th><code>flat</code></th><th><code>host</code></th></tr>
<tr>
<td>

```
~/Projects/src/
  owner/repo/
  owner/other-repo/
```

</td>
<td>

```
~/Projects/src/
  repo/
  other-repo/
```

</td>
<td>

```
~/Projects/src/
  github.com/owner/repo/
  gitlab.com/owner/other-repo/
```

</td>
</tr>
</table>

## Uninstall

```sh
rm ~/.local/bin/gx
```

Remove the `# gx` block from your shell config file (`~/.zshrc`, `~/.bashrc`, or `~/.config/fish/conf.d/gx.fish`).

## More

- [Use cases & examples](docs/use-cases.md) — real-world workflows, scripting recipes, and shell aliases
- [Contributing](CONTRIBUTING.md) — development setup and guidelines

## Acknowledgements

gx draws inspiration from these projects:

- **[ghq](https://github.com/x-motemen/ghq)** — the original structured repository manager. ghq pioneered the `host/owner/repo` directory layout and index-based project lookup that gx builds on.
- **[gclone](https://github.com/allonsy/gclone)** — a git clone helper with automatic directory organisation, shorthand URL parsing, and shell auto-cd. gx's clone workflow and shell integration owe a lot to gclone's approach.

## Acknowledgements

gx draws inspiration from these projects:

- **[ghq](https://github.com/x-motemen/ghq)** — the original structured repository manager. ghq pioneered the `host/owner/repo` directory layout and index-based project lookup that gx builds on.
- **[gclone](https://github.com/allonsy/gclone)** — a git clone helper with automatic directory organisation, shorthand URL parsing, and shell auto-cd. gx's clone workflow and shell integration owe a lot to gclone's approach.

## License

MIT
