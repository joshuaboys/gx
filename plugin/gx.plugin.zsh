# gx — git project manager shell integration
# Install: symlink or copy to ~/.oh-my-zsh/custom/plugins/gx/

# Capture plugin directory at source time (${0:A:h} only works at top level)
_GX_PLUGIN_DIR="${0:A:h}"

# Run the gx binary directly without eval — eliminates command injection risk
_gx_run() {
    if (( $+commands[gx] )); then
        command gx "$@"
    elif [ -f "$_GX_PLUGIN_DIR/../gx" ] && [ -x "$_GX_PLUGIN_DIR/../gx" ]; then
        "$_GX_PLUGIN_DIR/../gx" "$@"
    else
        bun run "$_GX_PLUGIN_DIR/../src/index.ts" "$@"
    fi
}

gx() {
    case "$1" in
        clone)
            # stdout = path (machine-readable), stderr = progress (human-readable)
            local output
            output=$(_gx_run clone "${@:2}")
            if [ -n "$output" ] && [ -d "$output" ]; then
                cd "$output"
            fi
            ;;
        ls|rebuild|config|open|init|--help|-h|--version|-v)
            _gx_run "$@"
            ;;
        resolve)
            _gx_run "$@"
            ;;
        "")
            _gx_run --help
            ;;
        *)
            # Default: jump to project (with fuzzy matching fallback)
            # stdout = path, stderr = fuzzy match info
            local target
            target=$(_gx_run resolve "$1")
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
    commands=(clone ls rebuild config resolve open init --help --version)

    if (( CURRENT == 2 )); then
        # First arg: commands + project names
        projects=($(_gx_run resolve --list 2>/dev/null))
        compadd "${commands[@]}" "${projects[@]}"
    elif (( CURRENT == 3 )) && [[ "${words[2]}" == "config" ]]; then
        compadd set
    elif (( CURRENT == 4 )) && [[ "${words[2]}" == "config" ]] && [[ "${words[3]}" == "set" ]]; then
        compadd projectDir defaultHost structure shallow similarityThreshold editor
    fi
}
(( $+functions[compdef] )) && compdef _gx gx
