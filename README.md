# gx

A fast git project manager for cloning, navigating, and organising repositories. Built with TypeScript and Bun.

## Features

- **Organised cloning** --- `gx clone user/repo` clones to a structured directory and cd's into it
- **Instant navigation** --- `gx <name>` jumps to any indexed project with tab completion
- **Fuzzy matching** --- misspell a project name and gx suggests the closest match
- **Editor integration** --- `gx open <name>` opens a project in your preferred editor
- **Agent scaffolding** --- `gx init` scaffolds `.claude/` configuration for AI-assisted development
- **Shell integration** --- native support for zsh, bash, and fish with completions

## Install

### Prerequisites

- [Bun](https://bun.sh) v1.0+

### Build and install

```sh
git clone https://github.com/joshuaboys/gx
cd gx
bun install
bun run build
```

Copy the binary somewhere on your `PATH`:

```sh
cp gx ~/.local/bin/
```

### Shell integration

Add one line to your shell config to enable `cd` on clone/jump and tab completion:

**zsh** (`~/.zshrc`):
```sh
eval "$(gx shell-init)"
```

**bash** (`~/.bashrc`):
```sh
eval "$(gx shell-init)"
```

**fish** (`~/.config/fish/conf.d/gx.fish`):
```fish
gx shell-init | source
```

Reload your shell and you're good to go.

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

### Configuration

```sh
gx config                         # show current config
gx config set projectDir ~/code   # change project directory
gx config set structure host      # use host/owner/repo layout
gx config set editor code         # set default editor
```

Config is stored at `~/.config/gx/config.json`.

### Rebuild index

```sh
gx rebuild
```

Rescans the project directory and rebuilds the project index.

## Directory structure

By default, gx uses a flat structure:

```
~/Projects/src/
  owner/repo/
  owner/other-repo/
```

Set `structure` to `host` for host-prefixed layout:

```
~/Projects/src/
  github.com/owner/repo/
  gitlab.com/owner/other-repo/
```

## Development

```sh
bun test           # run tests
bun run dev        # run from source
bun run build      # compile binary
```

## License

MIT
