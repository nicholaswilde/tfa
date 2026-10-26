_tfa() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="tfa"
                ;;
            tfa,add)
                cmd="tfa__subcmd__add"
                ;;
            tfa,audit)
                cmd="tfa__subcmd__audit"
                ;;
            tfa,completions)
                cmd="tfa__subcmd__completions"
                ;;
            tfa,delete)
                cmd="tfa__subcmd__delete"
                ;;
            tfa,export)
                cmd="tfa__subcmd__export"
                ;;
            tfa,generate)
                cmd="tfa__subcmd__generate"
                ;;
            tfa,help)
                cmd="tfa__subcmd__help"
                ;;
            tfa,import)
                cmd="tfa__subcmd__import"
                ;;
            tfa,instant)
                cmd="tfa__subcmd__instant"
                ;;
            tfa,list)
                cmd="tfa__subcmd__list"
                ;;
            tfa,man)
                cmd="tfa__subcmd__man"
                ;;
            tfa,move)
                cmd="tfa__subcmd__move"
                ;;
            tfa,namespace)
                cmd="tfa__subcmd__namespace"
                ;;
            tfa,password)
                cmd="tfa__subcmd__password"
                ;;
            tfa,qr)
                cmd="tfa__subcmd__qr"
                ;;
            tfa,rename)
                cmd="tfa__subcmd__rename"
                ;;
            tfa,show)
                cmd="tfa__subcmd__show"
                ;;
            tfa,status)
                cmd="tfa__subcmd__status"
                ;;
            tfa,watch)
                cmd="tfa__subcmd__watch"
                ;;
            tfa__subcmd__help,add)
                cmd="tfa__subcmd__help__subcmd__add"
                ;;
            tfa__subcmd__help,audit)
                cmd="tfa__subcmd__help__subcmd__audit"
                ;;
            tfa__subcmd__help,completions)
                cmd="tfa__subcmd__help__subcmd__completions"
                ;;
            tfa__subcmd__help,delete)
                cmd="tfa__subcmd__help__subcmd__delete"
                ;;
            tfa__subcmd__help,export)
                cmd="tfa__subcmd__help__subcmd__export"
                ;;
            tfa__subcmd__help,generate)
                cmd="tfa__subcmd__help__subcmd__generate"
                ;;
            tfa__subcmd__help,help)
                cmd="tfa__subcmd__help__subcmd__help"
                ;;
            tfa__subcmd__help,import)
                cmd="tfa__subcmd__help__subcmd__import"
                ;;
            tfa__subcmd__help,instant)
                cmd="tfa__subcmd__help__subcmd__instant"
                ;;
            tfa__subcmd__help,list)
                cmd="tfa__subcmd__help__subcmd__list"
                ;;
            tfa__subcmd__help,man)
                cmd="tfa__subcmd__help__subcmd__man"
                ;;
            tfa__subcmd__help,move)
                cmd="tfa__subcmd__help__subcmd__move"
                ;;
            tfa__subcmd__help,namespace)
                cmd="tfa__subcmd__help__subcmd__namespace"
                ;;
            tfa__subcmd__help,password)
                cmd="tfa__subcmd__help__subcmd__password"
                ;;
            tfa__subcmd__help,qr)
                cmd="tfa__subcmd__help__subcmd__qr"
                ;;
            tfa__subcmd__help,rename)
                cmd="tfa__subcmd__help__subcmd__rename"
                ;;
            tfa__subcmd__help,show)
                cmd="tfa__subcmd__help__subcmd__show"
                ;;
            tfa__subcmd__help,status)
                cmd="tfa__subcmd__help__subcmd__status"
                ;;
            tfa__subcmd__help,watch)
                cmd="tfa__subcmd__help__subcmd__watch"
                ;;
            tfa__subcmd__help__subcmd__namespace,create)
                cmd="tfa__subcmd__help__subcmd__namespace__subcmd__create"
                ;;
            tfa__subcmd__help__subcmd__namespace,delete)
                cmd="tfa__subcmd__help__subcmd__namespace__subcmd__delete"
                ;;
            tfa__subcmd__help__subcmd__namespace,list)
                cmd="tfa__subcmd__help__subcmd__namespace__subcmd__list"
                ;;
            tfa__subcmd__help__subcmd__namespace,rename)
                cmd="tfa__subcmd__help__subcmd__namespace__subcmd__rename"
                ;;
            tfa__subcmd__help__subcmd__namespace,set-default)
                cmd="tfa__subcmd__help__subcmd__namespace__subcmd__set__subcmd__default"
                ;;
            tfa__subcmd__help__subcmd__password,add)
                cmd="tfa__subcmd__help__subcmd__password__subcmd__add"
                ;;
            tfa__subcmd__help__subcmd__password,lock)
                cmd="tfa__subcmd__help__subcmd__password__subcmd__lock"
                ;;
            tfa__subcmd__help__subcmd__password,remove)
                cmd="tfa__subcmd__help__subcmd__password__subcmd__remove"
                ;;
            tfa__subcmd__help__subcmd__password,status)
                cmd="tfa__subcmd__help__subcmd__password__subcmd__status"
                ;;
            tfa__subcmd__help__subcmd__password,unlock)
                cmd="tfa__subcmd__help__subcmd__password__subcmd__unlock"
                ;;
            tfa__subcmd__help__subcmd__watch,all)
                cmd="tfa__subcmd__help__subcmd__watch__subcmd__all"
                ;;
            tfa__subcmd__namespace,create)
                cmd="tfa__subcmd__namespace__subcmd__create"
                ;;
            tfa__subcmd__namespace,delete)
                cmd="tfa__subcmd__namespace__subcmd__delete"
                ;;
            tfa__subcmd__namespace,help)
                cmd="tfa__subcmd__namespace__subcmd__help"
                ;;
            tfa__subcmd__namespace,list)
                cmd="tfa__subcmd__namespace__subcmd__list"
                ;;
            tfa__subcmd__namespace,rename)
                cmd="tfa__subcmd__namespace__subcmd__rename"
                ;;
            tfa__subcmd__namespace,set-default)
                cmd="tfa__subcmd__namespace__subcmd__set__subcmd__default"
                ;;
            tfa__subcmd__namespace__subcmd__help,create)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__create"
                ;;
            tfa__subcmd__namespace__subcmd__help,delete)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__delete"
                ;;
            tfa__subcmd__namespace__subcmd__help,help)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__help"
                ;;
            tfa__subcmd__namespace__subcmd__help,list)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__list"
                ;;
            tfa__subcmd__namespace__subcmd__help,rename)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__rename"
                ;;
            tfa__subcmd__namespace__subcmd__help,set-default)
                cmd="tfa__subcmd__namespace__subcmd__help__subcmd__set__subcmd__default"
                ;;
            tfa__subcmd__password,add)
                cmd="tfa__subcmd__password__subcmd__add"
                ;;
            tfa__subcmd__password,help)
                cmd="tfa__subcmd__password__subcmd__help"
                ;;
            tfa__subcmd__password,lock)
                cmd="tfa__subcmd__password__subcmd__lock"
                ;;
            tfa__subcmd__password,remove)
                cmd="tfa__subcmd__password__subcmd__remove"
                ;;
            tfa__subcmd__password,status)
                cmd="tfa__subcmd__password__subcmd__status"
                ;;
            tfa__subcmd__password,unlock)
                cmd="tfa__subcmd__password__subcmd__unlock"
                ;;
            tfa__subcmd__password__subcmd__help,add)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__add"
                ;;
            tfa__subcmd__password__subcmd__help,help)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__help"
                ;;
            tfa__subcmd__password__subcmd__help,lock)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__lock"
                ;;
            tfa__subcmd__password__subcmd__help,remove)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__remove"
                ;;
            tfa__subcmd__password__subcmd__help,status)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__status"
                ;;
            tfa__subcmd__password__subcmd__help,unlock)
                cmd="tfa__subcmd__password__subcmd__help__subcmd__unlock"
                ;;
            tfa__subcmd__watch,all)
                cmd="tfa__subcmd__watch__subcmd__all"
                ;;
            tfa__subcmd__watch,help)
                cmd="tfa__subcmd__watch__subcmd__help"
                ;;
            tfa__subcmd__watch__subcmd__help,all)
                cmd="tfa__subcmd__watch__subcmd__help__subcmd__all"
                ;;
            tfa__subcmd__watch__subcmd__help,help)
                cmd="tfa__subcmd__watch__subcmd__help__subcmd__help"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        tfa)
            opts="-C -n -h -V --config --storage --clipboard --clip-command --namespace --help --version generate instant watch add qr list status delete rename move show export import completions man namespace password audit help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__add)
            opts="-d -p -a -c -C -n -h --qr --digits --period --algorithm --counter --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --qr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --period)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --algorithm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --counter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__audit)
            opts="-C -n -h --json --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__completions)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__delete)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__export)
            opts="-C -n -h --unencrypted --password --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__generate)
            opts="-c -w -d -p -a -C -n -h --secret --copy --watch --counter --digits --period --algorithm --next --json --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --copy)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --counter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --period)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --algorithm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help)
            opts="generate instant watch add qr list status delete rename move show export import completions man namespace password audit help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__audit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__export)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__generate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__import)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__instant)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__man)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__move)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace)
            opts="list create rename delete set-default"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace__subcmd__rename)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__namespace__subcmd__set__subcmd__default)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password)
            opts="add remove lock unlock status"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password__subcmd__lock)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__password__subcmd__unlock)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__qr)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__rename)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__watch)
            opts="all"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__help__subcmd__watch__subcmd__all)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__import)
            opts="-C -n -h --dry-run --overwrite --skip --password --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__instant)
            opts="-d -p -a -c -w -C -n -h --secret --qr --digits --period --algorithm --counter --copy --watch --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --secret)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --qr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --period)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --algorithm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --counter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --copy)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__list)
            opts="-c -C -n -h --codes --json --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__man)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__move)
            opts="-t -C -n -h --to --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --to)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help list create rename delete set-default help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__create)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__delete)
            opts="-f -C -n -h --force --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help)
            opts="list create rename delete set-default help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__delete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__rename)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__help__subcmd__set__subcmd__default)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__list)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__rename)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__namespace__subcmd__set__subcmd__default)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help add remove lock unlock status help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__add)
            opts="-t -C -n -h --timeout --password --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help)
            opts="add remove lock unlock status help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__lock)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__status)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__help__subcmd__unlock)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__lock)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__remove)
            opts="-f -C -n -h --password --force --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__status)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__password__subcmd__unlock)
            opts="-C -n -h --password --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__qr)
            opts="-C -n -h --light --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --light)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__rename)
            opts="-C -n -h --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__show)
            opts="-r -C -n -h --raw --json --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__status)
            opts="-C -n -h --json --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__watch)
            opts="-a -f -c -C -n -h --all --filter --copy --config --storage --clipboard --clip-command --namespace --help all help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --filter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --copy)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__watch__subcmd__all)
            opts="-f -C -n -h --filter --config --storage --clipboard --clip-command --namespace --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --filter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --storage)
                    COMPREPLY=($(compgen -W "auto keyring file" -- "${cur}"))
                    return 0
                    ;;
                --clipboard)
                    COMPREPLY=($(compgen -W "system osc52" -- "${cur}"))
                    return 0
                    ;;
                --clip-command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --namespace)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__watch__subcmd__help)
            opts="all help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__watch__subcmd__help__subcmd__all)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tfa__subcmd__watch__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _tfa -o nosort -o bashdefault -o default tfa
else
    complete -F _tfa -o bashdefault -o default tfa
fi
