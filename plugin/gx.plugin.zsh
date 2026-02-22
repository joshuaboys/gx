# gx — git project manager shell integration
# Install: symlink or copy to ~/.oh-my-zsh/custom/plugins/gx/

# Capture plugin directory at source time (${0:A:h} only works at top level)
_GX_PLUGIN_DIR="${0:A:h}"

# Find the gx binary — compiled binary or bun dev
_gx_bin() {
    if (( $+commands[gx] )); then
        echo "${commands[gx]}"
    elif [ -f "$_GX_PLUGIN_DIR/../gx" ] && [ -x "$_GX_PLUGIN_DIR/../gx" ]; then
        echo "$_GX_PLUGIN_DIR/../gx"
    else
        echo "bun run $_GX_PLUGIN_DIR/../src/index.ts"
    fi
}

gx() {
    local bin
    bin=$(_gx_bin)

    case "$1" in
        clone)
            local output
            output=$(eval "$bin" clone "${@:2}" 2>/dev/null)
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            else
                eval "$bin" clone "${@:2}"
            fi
            ;;
        ls|rebuild|config|open|init|--help|-h|--version|-v)
            eval "$bin" "$@"
            ;;
        resolve)
            eval "$bin" "$@"
            ;;
        "")
            eval "$bin" --help
            ;;
        *)
            # Default: jump to project (with fuzzy matching fallback)
            # Capture stdout (path) and let stderr (fuzzy info) flow to terminal
            local target
            target=$(eval "$bin" resolve "$1")
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
    local bin
    bin=$(_gx_bin)

    local -a commands projects
    commands=(clone ls rebuild config resolve open init --help --version)

    if (( CURRENT == 2 )); then
        # First arg: commands + project names
        projects=($(eval "$bin" resolve --list 2>/dev/null))
        compadd "${commands[@]}" "${projects[@]}"
    elif (( CURRENT == 3 )) && [[ "${words[2]}" == "config" ]]; then
        compadd set
    elif (( CURRENT == 4 )) && [[ "${words[2]}" == "config" ]] && [[ "${words[3]}" == "set" ]]; then
        compadd projectDir defaultHost structure shallow similarityThreshold
    fi
}
(( $+functions[compdef] )) && compdef _gx gx
