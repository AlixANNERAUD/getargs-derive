#compdef cc

autoload -U is-at-least

_cc() {
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
'*-I+[Header directories]:INCLUDE:_default' \
'*--include=[Header directories]:INCLUDE:_default' \
'*-O+[Optimization level]:OPT:(0 1 2 3)' \
'*--opt=[Optimization level]:OPT:(0 1 2 3)' \
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
':language -- Target language:(c c++)' \
'*::sources -- Source files:(main.c util.c)' \
&& ret=0
}

if [ "$funcstack[1]" = "_cc" ]; then
    _cc "$@"
else
    compdef _cc cc
fi
