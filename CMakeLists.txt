cmake_minimum_required(VERSION 3.0)

include(./cmake/macros.cmake)
project(libymfm C CXX)

header_directories(
    ./components/ymfm/src
    ./src
)

add_source_files(
    ./components/ymfm/src/ymfm.h
    ./components/ymfm/src/ymfm_adpcm.cpp
    ./components/ymfm/src/ymfm_adpcm.h
    ./components/ymfm/src/ymfm_fm.h
    ./components/ymfm/src/ymfm_fm.ipp
    ./components/ymfm/src/ymfm_misc.cpp
    ./components/ymfm/src/ymfm_misc.h
    ./components/ymfm/src/ymfm_opl.cpp
    ./components/ymfm/src/ymfm_opl.h
    ./components/ymfm/src/ymfm_opm.cpp
    ./components/ymfm/src/ymfm_opm.h
    ./components/ymfm/src/ymfm_opn.cpp
    ./components/ymfm/src/ymfm_opn.h
    ./components/ymfm/src/ymfm_opq.cpp
    ./components/ymfm/src/ymfm_opq.h
    ./components/ymfm/src/ymfm_opz.cpp
    ./components/ymfm/src/ymfm_opz.h
    ./components/ymfm/src/ymfm_pcm.cpp
    ./components/ymfm/src/ymfm_pcm.h
    ./components/ymfm/src/ymfm_ssg.cpp
    ./components/ymfm/src/ymfm_ssg.h
    ./src/wasmymfm.cpp
)

add_compile_flags(CXX
    -std=c++14
)

add_compile_flags(LD
    "-s DEMANGLE_SUPPORT=1"
    "-s ALLOW_MEMORY_GROWTH=1"
    "-s MODULARIZE=1"
    "-s TOTAL_MEMORY=32MB"
    "-s FILESYSTEM=0"
)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ../lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ../lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ../lib)

add_executable(${PROJECT_NAME} ${SOURCE_FILES})
