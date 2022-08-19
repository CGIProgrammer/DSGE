#!/bin/env bash
laptop="192.168.1.38"
build_target="debug"
exec_name="base_example"
args=""
run=0
debug="gdb"
overlay="mangohud"

for item in "$@" ; do
    if [ "$item" == "run" ]; then
        run=1
        args=$args" build"
    else
        args=$args" $item"
    fi
    if [ "$item" == "--release" ]; then
        build_target="release"
        debug="$overlay"
    fi
done
cargo $args || exit 1
if [ $run == 1 ] ; then
    exec_file="target/$build_target/$exec_name"
    if [ $build_target != "debug" ]; then
        strip "$exec_file"
    fi
    #echo "Создание каталога на ноуте"
    #ssh -i ~/.ssh/comp $laptop 'mkdir -p ~/Rust/dsge_vk' || exit 1
    #echo "Упаковка билда для отправки"
    tar -czf "build.tar.gz" "data" "$exec_file" || exit 1
    #echo "Отправка билда"
    scp -i ~/.ssh/comp "build.tar.gz" $laptop:'~/Rust/dsge_vk/' || exit 1
    rm "build.tar.gz"
    #echo "Распаковка и запуск билда"
    ssh -i ~/.ssh/comp $laptop 'cd ~/Rust/dsge_vk && rm -rf data/shaders/* && rm shader_cache/*.spv'
    ssh -i ~/.ssh/comp $laptop "cd "'~/Rust/dsge_vk'" && tar -xvf build.tar.gz && rm -f build.tar.gz && DISPLAY=:0 $debug './$exec_file'"
fi