# gx — git project manager (oh-my-zsh compatibility shim)
# Prefer adding to ~/.zshrc directly: eval "$(gx shell-init)"
#
# This file exists for users who already have gx installed as an
# oh-my-zsh custom plugin. It delegates to `gx shell-init zsh`.

_GX_PLUGIN_DIR="${0:A:h}"

# Find the gx binary and eval its shell integration
if (( $+commands[gx] )); then
    eval "$(command gx shell-init zsh)"
elif [ -f "$_GX_PLUGIN_DIR/../gx" ] && [ -x "$_GX_PLUGIN_DIR/../gx" ]; then
    eval "$("$_GX_PLUGIN_DIR/../gx" shell-init zsh)"
else
    eval "$(bun run "$_GX_PLUGIN_DIR/../src/index.ts" shell-init zsh)"
fi
