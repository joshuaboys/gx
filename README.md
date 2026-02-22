# gx

A fast git project manager for cloning, navigating, and organizing repositories. Built with TypeScript and Bun.

## Features

- **Organized cloning** --- `gx clone user/repo` clones to a structured directory and cd's into it
- **Instant navigation** --- `gx <name>` jumps to any indexed project with tab completion
- **Fuzzy matching** --- misspell a project name and gx suggests the closest match
- **Editor integration** --- `gx open <name>` opens a project in your preferred editor
- **Agent scaffolding** --- `gx init` scaffolds `.claude/` configuration for AI-assisted development
- **Shell integration** --- oh-my-zsh plugin for `cd`, completions, and shell functions

## Install

### Prerequisites

- [Bun](https://bun.sh) v1.0+
- zsh with [oh-my-zsh](https://ohmyz.sh)

### Build and install

```sh
git clone https://github.com/joshuaboys/gx
cd gx

# Build the binary
bun run build

# Install the zsh plugin
ln -s "$(pwd)/plugin" ~/.oh-my-zsh/custom/plugins/gx
```

Add `gx` to your oh-my-zsh plugins in `~/.zshrc`:

```sh
plugins=(git gx)
```

Reload your shell:

```sh
source ~/.zshrc
```

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
