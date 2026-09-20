//! `gx shell-init [shell]` — print shell integration code. Ported from
//! the TS `src/commands/shell-init.ts` and since extended (e.g. `doctor`
//! pass-through); the snapshot harness pins the exact output. The only
//! environment-specific token is `_GX_BIN`, which the shell wrappers use to
//! invoke the binary.

use crate::errors::{GxError, GxResult};

const SUPPORTED_SHELLS: [&str; 4] = ["zsh", "bash", "fish", "powershell"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shell {
    Zsh,
    Bash,
    Fish,
    Pwsh,
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

fn parse_shell_name(name: &str) -> Option<Shell> {
    let trimmed = name.trim();
    let normalized = trimmed.replace('\\', "/");
    let file = std::path::Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed);
    let file = file
        .strip_suffix(".exe")
        .or_else(|| file.strip_suffix(".EXE"))
        .unwrap_or(file);
    match file.to_ascii_lowercase().as_str() {
        "zsh" => Some(Shell::Zsh),
        "bash" => Some(Shell::Bash),
        "fish" => Some(Shell::Fish),
        "pwsh" | "powershell" => Some(Shell::Pwsh),
        _ => None,
    }
}

fn detect_shell() -> Option<Shell> {
    if let Ok(name) = std::env::var("GX_SHELL_OVERRIDE") {
        if let Some(shell) = parse_shell_name(&name) {
            return Some(shell);
        }
    }
    if let Ok(shell) = std::env::var("SHELL") {
        if let Some(s) = parse_shell_name(&shell) {
            return Some(s);
        }
    }
    // PowerShell does not set $SHELL. PSModulePath is set in powershell/pwsh.
    let shell_set = std::env::var("SHELL").ok().is_some_and(|s| !s.is_empty());
    if !shell_set && std::env::var_os("PSModulePath").is_some() {
        return Some(Shell::Pwsh);
    }
    None
}

pub fn shell_init(shell_arg: Option<&str>) -> GxResult<()> {
    let shell = match shell_arg {
        Some(arg) => parse_shell_name(arg).ok_or_else(|| {
            GxError::command(format!(
                "Unsupported shell: {arg}\nSupported shells: {}",
                SUPPORTED_SHELLS.join(", ")
            ))
        })?,
        None => detect_shell().ok_or_else(|| {
            GxError::command(format!(
                "Could not detect shell from $SHELL\nSpecify one explicitly: gx shell-init <{}>",
                SUPPORTED_SHELLS.join("|")
            ))
        })?,
    };

    let bin = resolve_gx_bin();
    let bin = match shell {
        Shell::Pwsh => bin.replace('\'', "''"),
        _ => bin,
    };
    let template = match shell {
        Shell::Zsh => ZSH,
        Shell::Bash => BASH,
        Shell::Fish => FISH,
        Shell::Pwsh => POWERSHELL,
    };
    println!("{}", template.replace("__GX_BIN__", &bin));
    Ok(())
}

const ZSH: &str = r##"# gx — git project manager shell integration
# Add to ~/.zshrc: eval "$(gx shell-init)"

_GX_BIN="__GX_BIN__"

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

_GX_BIN="__GX_BIN__"

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

set -g _GX_BIN "__GX_BIN__"

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

const POWERSHELL: &str = r##"# gx — git project manager shell integration
# Add to $PROFILE: Invoke-Expression (& gx shell-init powershell | Out-String)

$script:_GX_BIN = '__GX_BIN__'

function global:gx {
    if ($args.Count -eq 0) {
        & $script:_GX_BIN --help
        return
    }

    $cmd = [string]$args[0]
    switch ($cmd) {
        'clone' {
            $output = & $script:_GX_BIN @args
            if ($LASTEXITCODE -ne 0) { return }
            $line = $output | Select-Object -Last 1
            if ($line -and (Test-Path -LiteralPath $line -PathType Container)) {
                Set-Location -LiteralPath $line
            }
        }
        'resume' {
            $output = & $script:_GX_BIN @args
            if ($LASTEXITCODE -ne 0) { return $LASTEXITCODE }
            $line = $output | Select-Object -Last 1
            if ($line -and (Test-Path -LiteralPath $line -PathType Container)) {
                Set-Location -LiteralPath $line
            } else {
                return 1
            }
        }
        { $_ -in 'ls','recent','rebuild','config','open','init','index','doctor','shell-init','resolve','--help','-h','--version','-v' } {
            & $script:_GX_BIN @args
        }
        default {
            $target = & $script:_GX_BIN resolve $cmd
            if ($LASTEXITCODE -ne 0) { return $LASTEXITCODE }
            $line = $target | Select-Object -Last 1
            if ($line -and (Test-Path -LiteralPath $line -PathType Container)) {
                Set-Location -LiteralPath $line
            } else {
                return 1
            }
            if ($args.Count -ge 2 -and $args[1] -eq 'wt') {
                $wt = Get-Command wt -ErrorAction SilentlyContinue
                if (-not $wt) {
                    [Console]::Error.WriteLine('gx: wt (worktrunk) not found on PATH')
                    [Console]::Error.WriteLine('Install: https://worktrunk.dev')
                    return 1
                }
                if ($args.Count -gt 2) {
                    & $wt.Source @($args[2..($args.Count - 1)])
                } else {
                    & $wt.Source
                }
            }
        }
    }
}

Register-ArgumentCompleter -CommandName gx -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)

    $commands = @(
        'clone','ls','recent','resume','rebuild','config','resolve',
        'open','init','index','doctor','shell-init','--help','--version','-h','-v'
    )
    $elements = @($commandAst.CommandElements | ForEach-Object { $_.Extent.Text })

    if ($elements.Count -le 2) {
        $projects = @(& $script:_GX_BIN resolve --list 2>$null)
        ($commands + $projects) | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }

    $first = $elements[1].Trim('"').Trim("'")
    if ($first -eq 'config') {
        if ($elements.Count -eq 3) {
            @('set') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
            return
        }
        if ($elements.Count -ge 4 -and $elements[2].Trim('"').Trim("'") -eq 'set') {
            @(
                'projectDir','defaultHost','structure','shallow',
                'similarityThreshold','editor'
            ) | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
            return
        }
    }

    if ($commands -notcontains $first) {
        @('wt') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shell_name_posix() {
        assert_eq!(parse_shell_name("zsh"), Some(Shell::Zsh));
        assert_eq!(parse_shell_name("/bin/bash"), Some(Shell::Bash));
        assert_eq!(parse_shell_name("/usr/bin/fish"), Some(Shell::Fish));
    }

    #[test]
    fn parse_shell_name_powershell() {
        assert_eq!(parse_shell_name("powershell"), Some(Shell::Pwsh));
        assert_eq!(parse_shell_name("pwsh"), Some(Shell::Pwsh));
        assert_eq!(
            parse_shell_name(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"),
            Some(Shell::Pwsh)
        );
        assert_eq!(parse_shell_name("pwsh.exe"), Some(Shell::Pwsh));
    }

    #[test]
    fn parse_shell_name_unknown() {
        assert_eq!(parse_shell_name("cmd"), None);
        assert_eq!(parse_shell_name("tcsh"), None);
    }
}
