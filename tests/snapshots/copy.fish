complete -c copy -s o -s d -l output -l destination -d 'Output file, `-` for "stdout"' -r
complete -c copy -l color -d 'When to use [colors]: auto or not' -r -f -a 'auto always never'
complete -c copy -s s -l size -d 'Size in bytes' -r
complete -c copy -s f -l force -d 'Overwrite existing files'
complete -c copy -s v -l verbose -d 'More output, repeat for more'
complete -c copy -s h -l help -d 'Print help (see more with \'--help\')'
complete -c copy -s V -l version -d 'Print version'
complete -c copy -a 'fast safe' -d 'Copy mode'
