_vcs() {
    local cur prev opts cmd start index
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd="vcs"
    start=0
    for (( index = 1; index < COMP_CWORD; index++ )); do
        case "${cmd},${COMP_WORDS[index]}" in
            "vcs,commit"|"vcs,ci")
                cmd="vcs__commit"
                start=${index}
                ;;
            "vcs,remote")
                cmd="vcs__remote"
                start=${index}
                ;;
            "vcs,debug")
                cmd="vcs__debug"
                start=${index}
                ;;
            "vcs__remote,add")
                cmd="vcs__remote__add"
                start=${index}
                ;;
            "vcs__remote,remove"|"vcs__remote,rm")
                cmd="vcs__remote__remove"
                start=${index}
                ;;
            "vcs__remote,show")
                cmd="vcs__remote__show"
                start=${index}
                ;;
        esac
    done

    case "${cmd}" in
        vcs)
            opts="-q --quiet -C --directory -h --help -V --version"

            case "${prev}" in
                -C|--directory)
                    COMPREPLY=($(compgen -f -- "${cur}"))
                    return 0
                    ;;
            esac

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -W "commit remote" -- "${cur}"))
            return 0
            ;;
        vcs__commit)
            opts="-m --message --color -h --help -q --quiet"

            case "${prev}" in
                -m|--message)
                    COMPREPLY=($(compgen -f -- "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
            esac

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        vcs__remote)
            opts="-h --help -q --quiet"

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -W "add remove rm show" -- "${cur}"))
            return 0
            ;;
        vcs__remote__add)
            opts="-p --protocol -h --help -q --quiet"

            case "${prev}" in
                -p|--protocol)
                    COMPREPLY=($(compgen -W "https ssh" -- "${cur}"))
                    return 0
                    ;;
            esac

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        vcs__remote__remove)
            opts="-h --help -q --quiet"

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        vcs__remote__show)
            opts="-h --help -q --quiet"

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
        vcs__debug)
            opts="-h --help -q --quiet"

            if [[ ${cur} == -* ]]; then
                COMPREPLY=($(compgen -W "${opts}" -- "${cur}"))
                return 0
            fi

            COMPREPLY=($(compgen -f -- "${cur}"))
            return 0
            ;;
    esac
}

complete -F _vcs -o bashdefault -o default vcs
