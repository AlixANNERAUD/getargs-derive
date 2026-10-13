complete -c cc -s I -l include -d 'Header directories' -r
complete -c cc -s O -l opt -d 'Optimization level' -r -f -a '0 1 2 3'
complete -c cc -s h -l help -d 'Print help'
complete -c cc -a 'c c++' -d 'Target language'
complete -c cc -a 'main.c util.c' -d 'Source files'
