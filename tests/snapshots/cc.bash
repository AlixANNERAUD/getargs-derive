_cc() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="-I --include -O --opt -h --help"

    case "${prev}" in
        -I|--include)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        -O|--opt)
            COMPREPLY=($(compgen -W "0 1 2 3" -- "${cur}"))
            return 0
            ;;
    esac

    if [[ ${cur} == -* ]]; then
        COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
        return 0
    fi

    local i position=0
    for (( i = 1; i < COMP_CWORD; i++ )); do
        case "${COMP_WORDS[i]}" in
            -I|--include) (( i++ )) ;;
            -O|--opt) (( i++ )) ;;
            -*) ;;
            *) (( position++ )) ;;
        esac
    done
    (( position > 1 )) && position=1
    case "${position}" in
        0)
            COMPREPLY=($(compgen -W "c c++" -- "${cur}"))
            return 0
            ;;
        1)
            COMPREPLY=($(compgen -W "main.c util.c" -- "${cur}"))
            return 0
            ;;
    esac

    COMPREPLY=($(compgen -f -- "${cur}"))
    return 0
}

complete -F _cc -o bashdefault -o default cc
