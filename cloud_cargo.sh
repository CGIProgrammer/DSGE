#!/bin/bash
proj_name="dsge_vk"
bin_name="base_example"
cmd=$1
target=$2
host="kompukter.keenetic.link"
proj_dir="/home/ivan/Rust/$proj_name"
execfile="$proj_dir/target/$target/$bin_name"
port=7346
key=~/.ssh/acer
cargo_path='$HOME/.cargo/bin/cargo'
features="use_image"

if [[ "$features" ]] ; then
   features="--features $features"
fi

echo "Сжатие исходников"
tar -czf src.tar.gz Cargo.toml src #data
echo "Отправка исходников (ssh -i $key -p $port $host)"
scp -i $key -P $port src.tar.gz $host:"$proj_dir"

case "$target" in
 "" )
    echo "Компиляция отладочной версии"
    target="debug"
    execfile="$proj_dir/target/$target/$bin_name"
    build_flag=""
 ;;
 
 "debug" )
    echo "Компиляция отладочной версии"
    build_flag=""
 ;;

 "release" )
    echo "Компиляция релизной версии"
    build_flag="--release"
 ;;*)

 echo "Неправильная цель сборки '$target'"
 exit 1
esac

ssh -i $key -p $port $host "cd $proj_dir && tar -xf src.tar.gz && rm src.tar.gz && $cargo_path build $features $build_flag --bin $bin_name"
echo "Сборка завершена."

#if [[ "$cmd" == "run" ]]; then
#   echo "Оптимизация размера файла."
#   ssh -i $key -p $port $host "cd ~/Rust/$proj_name/ && strip $execfile && upx $execfile"
#fi
echo "Сжатие файла"
ssh -i $key -p $port $host "cd ~/Rust/$proj_name/ && strip $execfile && tar -czf executable.tar.gz `relpath $execfile`"

echo "Скачивание исполняемого файла."
mkdir -p "target/$target/" 2> /dev/null
#scp -i $key -P $port $host:"$execfile" "target/$target/"
rm ./executable.tar.gz 2> /dev/null
scp -i $key -P $port $host:'~/Rust/'"$proj_name/executable.tar.gz" "./"

echo "Распаковка"
tar -xvf ./executable.tar.gz || exit
if [[ $cmd == "run" ]]; then
   echo "Запуск."
   rm shader_cache/*.spv 2> /dev/null
   mangohud "target/$target/$bin_name"
fi
