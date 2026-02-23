const SUPPORTED_SHELLS = ["zsh", "bash", "fish"] as const;
type Shell = (typeof SUPPORTED_SHELLS)[number];

function detectShell(): Shell | null {
  const shell = process.env.SHELL ?? "";
  if (shell.endsWith("/zsh")) return "zsh";
  if (shell.endsWith("/bash")) return "bash";
  if (shell.endsWith("/fish")) return "fish";
  return null;
}

function zshInit(): string {
  return `# gx — git project manager shell integration
# Add to ~/.zshrc: eval "$(gx shell-init)"

gx() {
    case "$1" in
        clone)
            local output
            output=$(command gx clone "\${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
        ls|rebuild|config|open|init|shell-init|--help|-h|--version|-v)
            command gx "$@"
            ;;
        resolve)
            command gx "$@"
            ;;
        "")
            command gx --help
            ;;
        *)
            local target
            target=$(command gx resolve "$1")
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target"
            else
                return 1
            fi
            ;;
    esac
}

# Tab completion
_gx() {
    local -a commands projects
    commands=(clone ls rebuild config resolve open init shell-init --help --version)

    if (( CURRENT == 2 )); then
        projects=($(command gx resolve --list 2>/dev/null))
        compadd "\${commands[@]}" "\${projects[@]}"
    elif (( CURRENT == 3 )) && [[ "\${words[2]}" == "config" ]]; then
        compadd set
    elif (( CURRENT == 4 )) && [[ "\${words[2]}" == "config" ]] && [[ "\${words[3]}" == "set" ]]; then
        compadd projectDir defaultHost structure shallow similarityThreshold editor
    fi
}
(( $+functions[compdef] )) && compdef _gx gx`;
}

function bashInit(): string {
  return `# gx — git project manager shell integration
# Add to ~/.bashrc: eval "$(gx shell-init)"

gx() {
    case "$1" in
        clone)
            local output
            output=$(command gx clone "\${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
        ls|rebuild|config|open|init|shell-init|--help|-h|--version|-v)
            command gx "$@"
            ;;
        resolve)
            command gx "$@"
            ;;
        "")
            command gx --help
            ;;
        *)
            local target
            target=$(command gx resolve "$1")
            if [ -n "$target" ] && [ -d "$target" ]; then
                cd "$target"
            else
                return 1
            fi
            ;;
    esac
}

# Tab completion
_gx_completions() {
    local cur="\${COMP_WORDS[COMP_CWORD]}"
    local prev="\${COMP_WORDS[COMP_CWORD-1]}"

    if [ "\$COMP_CWORD" -eq 1 ]; then
        local commands="clone ls rebuild config resolve open init shell-init --help --version"
        local projects
        projects=$(command gx resolve --list 2>/dev/null)
        COMPREPLY=($(compgen -W "$commands $projects" -- "$cur"))
    elif [ "\$COMP_CWORD" -eq 2 ] && [ "$prev" = "config" ]; then
        COMPREPLY=($(compgen -W "set" -- "$cur"))
    elif [ "\$COMP_CWORD" -eq 3 ] && [ "\${COMP_WORDS[1]}" = "config" ] && [ "$prev" = "set" ]; then
        COMPREPLY=($(compgen -W "projectDir defaultHost structure shallow similarityThreshold editor" -- "$cur"))
    fi
}
complete -F _gx_completions gx`;
}

function fishInit(): string {
  return `# gx — git project manager shell integration
# Add to ~/.config/fish/conf.d/gx.fish: gx shell-init | source

function gx
    switch $argv[1]
        case clone
            set -l output (command gx clone $argv[2..])
            if test -n "$output" -a -d "$output"
                cd $output
            end
        case ls rebuild config open init shell-init --help -h --version -v resolve
            command gx $argv
        case ''
            command gx --help
        case '*'
            set -l target (command gx resolve $argv[1])
            if test -n "$target" -a -d "$target"
                cd $target
            else
                return 1
            end
    end
end

# Tab completion
complete -c gx -f
complete -c gx -n "__fish_use_subcommand" -a "clone ls rebuild config resolve open init shell-init --help --version"
complete -c gx -n "__fish_use_subcommand" -a "(command gx resolve --list 2>/dev/null)"
complete -c gx -n "__fish_seen_subcommand_from config" -a "set"
complete -c gx -n "__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set" -a "projectDir defaultHost structure shallow similarityThreshold editor"`;
}

export function shellInit(shellArg?: string): void {
  let shell: Shell | null;

  if (shellArg) {
    if (!SUPPORTED_SHELLS.includes(shellArg as Shell)) {
      console.error(`Unsupported shell: ${shellArg}`);
      console.error(`Supported shells: ${SUPPORTED_SHELLS.join(", ")}`);
      process.exit(1);
    }
    shell = shellArg as Shell;
  } else {
    shell = detectShell();
    if (!shell) {
      console.error("Could not detect shell from $SHELL");
      console.error(`Specify one explicitly: gx shell-init <${SUPPORTED_SHELLS.join("|")}>`);
      process.exit(1);
    }
  }

  switch (shell) {
    case "zsh":
      console.log(zshInit());
      break;
    case "bash":
      console.log(bashInit());
      break;
    case "fish":
      console.log(fishInit());
      break;
  }
}
