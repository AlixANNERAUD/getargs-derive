#compdef vcs

autoload -U is-at-least

_vcs() {
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
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
'(-C --directory)-C+[Working directory]:DIRECTORY:_default' \
'(-C --directory)--directory=[Working directory]:DIRECTORY:_default' \
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-V --version)-V[Print version]' \
'(-V --version)--version[Print version]' \
':command:_vcs_commands' \
'*::arg:->args' \
&& ret=0
    case $state in
    (args)
        curcontext="${curcontext%:*:*}:vcs-command-$words[1]:"
        case $words[1] in
        (commit|ci)
            _vcs__commit && ret=0
            ;;
        (remote)
            _vcs__remote && ret=0
            ;;
        (debug)
            _vcs__debug && ret=0
            ;;
        esac
        ;;
    esac
    return ret
}

_vcs_commands() {
    local commands; commands=(
        'commit:Commit changes'
        'remote:Manage remotes'
    )
    _describe -t commands 'vcs commands' commands "$@"
}

_vcs__commit() {
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
'*-m+[Commit message]:MESSAGE:_default' \
'*--message=[Commit message]:MESSAGE:_default' \
'(--color)--color=[When to use colors]:COLOR:(auto always never)' \
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
&& ret=0
    return ret
}

_vcs__remote() {
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
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
':command:_vcs__remote_commands' \
'*::arg:->args' \
&& ret=0
    case $state in
    (args)
        curcontext="${curcontext%:*:*}:vcs__remote-command-$words[1]:"
        case $words[1] in
        (add)
            _vcs__remote__add && ret=0
            ;;
        (remove|rm)
            _vcs__remote__remove && ret=0
            ;;
        (show)
            _vcs__remote__show && ret=0
            ;;
        esac
        ;;
    esac
    return ret
}

_vcs__remote_commands() {
    local commands; commands=(
        'add:Add a remote'
        'remove:Remove a remote'
        'rm:Remove a remote'
        'show:Show a remote'
    )
    _describe -t commands 'vcs remote commands' commands "$@"
}

_vcs__remote__add() {
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
'(-p --protocol)-p+[Protocol to use]:PROTOCOL:(https ssh)' \
'(-p --protocol)--protocol=[Protocol to use]:PROTOCOL:(https ssh)' \
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
':name -- Name of the remote:_default' \
&& ret=0
    return ret
}

_vcs__remote__remove() {
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
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
':name:_default' \
&& ret=0
    return ret
}

_vcs__remote__show() {
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
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
'::name -- Remote to show:_default' \
&& ret=0
    return ret
}

_vcs__debug() {
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
'(-h --help)-h[Print help]' \
'(-h --help)--help[Print help]' \
'(-q --quiet)-q[Print less output]' \
'(-q --quiet)--quiet[Print less output]' \
&& ret=0
    return ret
}

if [ "$funcstack[1]" = "_vcs" ]; then
    _vcs "$@"
else
    compdef _vcs vcs
fi
