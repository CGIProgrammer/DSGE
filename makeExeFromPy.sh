#/usr/bin/bash
cython3 --embed "$1.py"
gcc `pkgconf --cflags python3` "$1.c" `pkgconf --libs python3` -o $1
rm "$1.c"