//! `gx shell-init [shell]` — print shell integration code. Ported from
//! `src/commands/shell-init.ts`. The emitted code is byte-identical to the TS
//! output (the snapshot harness enforces this); the only environment-specific
//! token is `_GX_BIN`, which the shell wrappers use to invoke the binary.

use crate::errors::{GxError, GxResult};

const SUPPORTED_SHELLS: [&str; 3] = ["zsh", "bash", "fish"];

#[derive(Clone, Copy)]
enum Shell {
    Zsh,
    Bash,
    Fish,
}

/// Resolve the absolute path to embed as `_GX_BIN`. Unlike the TS port (which
/// PATH-looks-up `gx` because Bun compiled binaries report an unreliable
/// `argv[0]`), the Rust binary can ask the OS for its own path via
/// `current_exe()`, which is always correct and never warns. The value is
/// scrubbed to `<BIN>` in snapshots, so this stays parity-clean.
fn resolve_gx_bin() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "gx".to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn fish_quote(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn detect_shell() -> Option<Shell> {
    if let Ok(name) = std::env::var("GX_SHELL_OVERRIDE") {
        if name.ends_with("/zsh") || name == "zsh" {
            return Some(Shell::Zsh);
        }
        if name.ends_with("/bash") || name == "bash" {
            return Some(Shell::Bash);
        }
        if name.ends_with("/fish") || name == "fish" {
            return Some(Shell::Fish);
        }
    }
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.ends_with("/zsh") {
        return Some(Shell::Zsh);
    }
    if shell.ends_with("/bash") {
        return Some(Shell::Bash);
    }
    if shell.ends_with("/fish") {
        return Some(Shell::Fish);
    }
    None
}

pub fn shell_init(shell_arg: Option<&str>) -> GxResult<()> {
    let shell = match shell_arg {
        Some(arg) => {
            if !SUPPORTED_SHELLS.contains(&arg) {
                return Err(GxError::command(format!(
                    "Unsupported shell: {arg}\nSupported shells: {}",
                    SUPPORTED_SHELLS.join(", ")
                )));
            }
            match arg {
                "zsh" => Shell::Zsh,
                "bash" => Shell::Bash,
                _ => Shell::Fish,
            }
        }
        None => detect_shell().ok_or_else(|| {
            GxError::command(format!(
                "Could not detect shell from $SHELL\nSpecify one explicitly: gx shell-init <{}>",
                SUPPORTED_SHELLS.join("|")
            ))
        })?,
    };

    let raw_bin = resolve_gx_bin();
    let bin = match shell {
        Shell::Fish => fish_quote(&raw_bin),
        Shell::Zsh | Shell::Bash => shell_quote(&raw_bin),
    };
    let template = match shell {
        Shell::Zsh => ZSH,
        Shell::Bash => BASH,
        Shell::Fish => FISH,
    };
    println!("{}", template.replace("__GX_BIN__", &bin));
    Ok(())
}

const ZSH: &str = r##"# gx — git project manager shell integration
# Add to ~/.zshrc: eval "$(gx shell-init)"

_GX_BIN=__GX_BIN__

gx() {
    case "$1" in
        clone)
            local output
            output=$("$_GX_BIN" clone "${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
        resume)
            local output
            output=$("$_GX_BIN" resume "${@:2}")
            local status=$?
            if [ "$status" -ne 0 ]; then
                return "$status"
            fi
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            else
                return 1
            fi
            ;;
        ls|recent|rebuild|config|open|init|index|doctor|shell-init|--help|-h|--version|-v)
            "$_GX_BIN" "$@"
            ;;
        resolve)
            "$_GX_BIN" "$@"
            ;;
        "")
            "$_GX_BIN" --help
            ;;
        *)
            local target
            target=$("$_GX_BIN" resolve "$1")
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target"
            else
                return 1
            fi
            if [ "$2" = "wt" ]; then
                if ! command -v wt >/dev/null 2>&1; then
                    echo "gx: wt (worktrunk) not found on PATH" >&2
                    echo "Install: brew install worktrunk" >&2
                    return 1
                fi
                shift 2
                wt "$@"
            fi
            ;;
    esac
}

# Tab completion
_gx() {
    local -a commands projects
    commands=(clone ls recent resume rebuild config resolve open init index doctor shell-init --help --version -h -v)

    if (( CURRENT == 2 )); then
        projects=($("$_GX_BIN" resolve --list 2>/dev/null))
        compadd "${commands[@]}" "${projects[@]}"
    elif (( CURRENT == 3 )); then
        case "${words[2]}" in
            clone|ls|recent|resume|rebuild|config|resolve|open|init|index|doctor|shell-init|--help|--version|-h|-v)
                if [[ "${words[2]}" == "config" ]]; then
                    compadd set
                fi
                ;;
            *)
                compadd wt
                ;;
        esac
    elif (( CURRENT == 4 )) && [[ "${words[2]}" == "config" ]] && [[ "${words[3]}" == "set" ]]; then
        compadd projectDir defaultHost structure shallow similarityThreshold editor
    fi
}
(( $+functions[compdef] )) && compdef _gx gx"##;

const BASH: &str = r##"# gx — git project manager shell integration
# Add to ~/.bashrc: eval "$(gx shell-init)"

_GX_BIN=__GX_BIN__

gx() {
    case "$1" in
        clone)
            local output
            output=$("$_GX_BIN" clone "${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
        resume)
            local output status
            output=$("$_GX_BIN" resume "${@:2}")
            status=$?
            if [ "$status" -ne 0 ]; then
                return "$status"
            fi
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            else
                return 1
            fi
            ;;
        ls|recent|rebuild|config|open|init|index|doctor|shell-init|--help|-h|--version|-v)
            "$_GX_BIN" "$@"
            ;;
        resolve)
            "$_GX_BIN" "$@"
            ;;
        "")
            "$_GX_BIN" --help
            ;;
        *)
            local target
            target=$("$_GX_BIN" resolve "$1")
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target"
            else
                return 1
            fi
            if [ "$2" = "wt" ]; then
                if ! command -v wt >/dev/null 2>&1; then
                    echo "gx: wt (worktrunk) not found on PATH" >&2
                    echo "Install: brew install worktrunk" >&2
                    return 1
                fi
                shift 2
                wt "$@"
            fi
            ;;
    esac
}

# Tab completion
_gx_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    if [ "$COMP_CWORD" -eq 1 ]; then
        local commands="clone ls recent resume rebuild config resolve open init index doctor shell-init --help --version -h -v"
        local projects
        projects=$("$_GX_BIN" resolve --list 2>/dev/null)
        COMPREPLY=($(compgen -W "$commands $projects" -- "$cur"))
    elif [ "$COMP_CWORD" -eq 2 ]; then
        case "${COMP_WORDS[1]}" in
            clone|ls|recent|resume|rebuild|config|resolve|open|init|index|doctor|shell-init|--help|--version|-h|-v)
                if [ "${COMP_WORDS[1]}" = "config" ]; then
                    COMPREPLY=($(compgen -W "set" -- "$cur"))
                fi
                ;;
            *)
                COMPREPLY=($(compgen -W "wt" -- "$cur"))
                ;;
        esac
    elif [ "$COMP_CWORD" -eq 3 ] && [ "${COMP_WORDS[1]}" = "config" ] && [ "$prev" = "set" ]; then
        COMPREPLY=($(compgen -W "projectDir defaultHost structure shallow similarityThreshold editor" -- "$cur"))
    fi
}
complete -F _gx_completions gx"##;

const FISH: &str = r##"# gx — git project manager shell integration
# Add to ~/.config/fish/conf.d/gx.fish: gx shell-init | source

set -g _GX_BIN __GX_BIN__

function gx
    switch $argv[1]
        case clone
            set -l output ($_GX_BIN clone $argv[2..])
            if test -n "$output" -a -d "$output"
                cd "$output"
            end
        case resume
            set -l output ($_GX_BIN resume $argv[2..])
            set -l cmd_status $status
            if test $cmd_status -ne 0
                return $cmd_status
            end
            if test -n "$output" -a -d "$output"
                cd "$output"
            else
                return 1
            end
        case ls recent rebuild config open init index doctor shell-init --help -h --version -v resolve
            $_GX_BIN $argv
        case ''
            $_GX_BIN --help
        case '*'
            set -l target ($_GX_BIN resolve $argv[1])
            if test -n "$target" -a -d "$target"
                cd "$target"
            else
                return 1
            end
            if test "$argv[2]" = "wt"
                if not command -v wt >/dev/null 2>&1
                    echo "gx: wt (worktrunk) not found on PATH" >&2
                    echo "Install: brew install worktrunk" >&2
                    return 1
                end
                wt $argv[3..]
            end
    end
end

# Tab completion
complete -c gx -f
complete -c gx -n "__fish_use_subcommand" -a "clone ls recent resume rebuild config resolve open init index doctor shell-init --help --version -h -v"
complete -c gx -n "__fish_use_subcommand" -a "($_GX_BIN resolve --list 2>/dev/null)"
complete -c gx -n "__fish_seen_subcommand_from config" -a "set"
complete -c gx -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -a "projectDir defaultHost structure shallow similarityThreshold editor"
complete -c gx -n "not __fish_seen_subcommand_from clone ls recent resume rebuild config resolve open init index doctor shell-init --help -h --version -v; and test (count (commandline -opc)) -eq 2" -a "wt""##;
