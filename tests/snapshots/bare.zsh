#compdef bare

autoload -U is-at-least

_bare() {
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
':value:_default' \
&& ret=0
}

if [ "$funcstack[1]" = "_bare" ]; then
    _bare "$@"
else
    compdef _bare bare
fi
