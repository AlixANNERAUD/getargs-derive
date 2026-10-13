_copy() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="-o -d --output --destination --color -s --size -f --force -v --verbose -h --help -V --version"

    case "${prev}" in
        -o|-d|--output|--destination)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        --color)
            COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
            return 0
            ;;
        -s|--size)
            COMPREPLY=($(compgen -f -- "${cur}"))
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
            -o|-d|--output|--destination) (( i++ )) ;;
            --color) (( i++ )) ;;
            -s|--size) (( i++ )) ;;
            -*) ;;
            *) (( position++ )) ;;
        esac
    done
    case "${position}" in
        1)
            COMPREPLY=($(compgen -W "fast safe" -- "${cur}"))
            return 0
            ;;
    esac

    COMPREPLY=($(compgen -f -- "${cur}"))
    return 0
}

complete -F _copy -o bashdefault -o default copy
