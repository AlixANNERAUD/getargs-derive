#compdef copy

autoload -U is-at-least

_copy() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'(-o -d --output --destination)-o+[Output file, \`-\` for "stdout"]:FILE:_default' \
'(-o -d --output --destination)-d+[Output file, \`-\` for "stdout"]:FILE:_default' \
'(-o -d --output --destination)--output=[Output file, \`-\` for "stdout"]:FILE:_default' \
'(-o -d --output --destination)--destination=[Output file, \`-\` for "stdout"]:FILE:_default' \
'(--color)--color=[When to use \[colors\]\: auto or not]:COLOR:(auto always never)' \
'(-s --size)-s+[Size in bytes]:SIZE:_default' \
'(-s --size)--size=[Size in bytes]:SIZE:_default' \
'(-f --force)-f[Overwrite existing files]' \
'(-f --force)--force[Overwrite existing files]' \
'*-v[More output, repeat for more]' \
'*--verbose[More output, repeat for more]' \
'(-h --help)-h[Print help (see more with '\''--help'\'')]' \
'(-h --help)--help[Print help (see more with '\''--help'\'')]' \
'(-V --version)-V[Print version]' \
'(-V --version)--version[Print version]' \
':input -- Input file:_default' \
'::mode -- Copy mode:(fast safe)' \
'::rest -- Arguments passed through:_default' \
&& ret=0
}

if [ "$funcstack[1]" = "_copy" ]; then
    _copy "$@"
else
    compdef _copy copy
fi
