cmake_minimum_required(VERSION 3.10)
project(dxil-signing)

set(CMAKE_CXX_STANDARD 17)

add_compile_options(
    "$<$<CONFIG:Debug>:/MTd>"
    "$<$<CONFIG:RelWithDebInfo>:/MT>"
    "$<$<CONFIG:Release>:/MT>"
    "$<$<CONFIG:MinSizeRel>:/MT>"
)

add_executable(dxil-signing main.cpp)