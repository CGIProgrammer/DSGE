cmake_minimum_required(VERSION 3.16)

project(DSGE VERSION 1.0.0 LANGUAGES C)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_C_FLAGS "-Wall -Wextra -ffunction-sections -Wl,--gc-sections -fno-asynchronous-unwind-tables -Wl,--strip-all -flto")
set(CMAKE_C_FLAGS_RELEASE "-Os")

#SET(GCC_COMPILE_FLAGS "-Os")
#SET(GCC_LINK_FLAGS    "-ffunction-sections -Wl,--gc-sections -fno-asynchronous-unwind-tables -Wl,--strip-all -flto")

set(SOURCE_DIR ./src)
set(STRUCTURES_DIR ${SOURCE_DIR}/structures)
set(ENGINE 
    ${SOURCE_DIR}/linalg.c
    ${SOURCE_DIR}/memmanager.c
    ${SOURCE_DIR}/glad.c
    ${SOURCE_DIR}/io.c
    ${STRUCTURES_DIR}/components/camera_component.c
    ${STRUCTURES_DIR}/components/light_component.c
    ${STRUCTURES_DIR}/dict.c
    ${STRUCTURES_DIR}/framebuffer.c
    ${STRUCTURES_DIR}/game_object.c
    ${STRUCTURES_DIR}/list.c
    ${STRUCTURES_DIR}/material.c
    ${STRUCTURES_DIR}/mesh.c
    ${STRUCTURES_DIR}/shader.c
    ${STRUCTURES_DIR}/texture.c
    ${STRUCTURES_DIR}/scene.c
)
set(SOURCE_EXE ${SOURCE_DIR}/gl_tester_2.c)
set(SOURCE_LIB ${ENGINE})			# Тоже самое, но для библиотеки

#find_file(SDL2_INCLUDE_DIR NAME SDL.h HINTS SDL2)
#find_library(SDL2_LIBRARY NAME SDL2)
INCLUDE(FindPkgConfig)
PKG_SEARCH_MODULE(SDL2 REQUIRED sdl2)
INCLUDE_DIRECTORIES(${SDL2_INCLUDE_DIRS} ./inc)

add_library(dsge STATIC ${SOURCE_LIB})	# Создание статической библиотеки с именем foo
add_executable(dsge_demo ${SOURCE_EXE})	# Создает исполняемый файл с именем main

#target_include_directories(dsge_demo PUBLIC ./inc)
target_link_libraries(dsge_demo dsge ${SDL2_LIBRARIES} m dl GL)	# Линковка программы с библиотекой
