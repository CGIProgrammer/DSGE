#!/bin/bash
if [[ $1 == "-d" ]] ; then
    data="data"
    echo "Копирование с данными."
else
    echo "Копирование без данных."
fi
echo "Сжатие." &&
echo "cd `pwd` && tar -czf dsge_vk.tgz std140-0.2.6 src Cargo.toml $data" &&
compssh "cd `pwd` && tar -czvf dsge_vk.tgz std140-0.2.6 src Cargo.toml $data" &&
echo "Копирование." &&
compscp @"`pwd`/dsge_vk.tgz" ./ &&
echo "Распаковка." &&
tar -xvf dsge_vk.tgz &
tar_pid=$!
compssh "rm \"`pwd`/dsge_vk.tgz\" > /dev/null 2> /dev/null"
wait $tar_pid
