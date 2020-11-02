#!/bin/bash

clear && gcc -Wall \
    ./src/*.c \
    ./src/structures/components/*.c \
    ./src/structures/*.c \
    -Iinc -Iinc/structures -Iinc/structures/components \
    -Wno-unused-result -Wno-shift-count-overflow \
    -Os \
    -lm -lGL -ldl \
    `sdl2-config --cflags` \
    `sdl2-config --libs` \
    -o gl_tester \
    -ffunction-sections -Wl,--gc-sections -fno-asynchronous-unwind-tables -Wl,--strip-all -flto
