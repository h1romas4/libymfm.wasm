cmake_minimum_required(VERSION 3.0)

project(ymfm C CXX)

include(./cmake/macros.cmake)

header_directories(
    ./components/ymfm/src
    ./src
)

add_source_files(
    ./components/ymfm/src/ymfm_adpcm.cpp
    ./components/ymfm/src/ymfm_misc.cpp
    ./components/ymfm/src/ymfm_opl.cpp
    ./components/ymfm/src/ymfm_opm.cpp
    ./components/ymfm/src/ymfm_opn.cpp
    ./components/ymfm/src/ymfm_opq.cpp
    ./components/ymfm/src/ymfm_opz.cpp
    ./components/ymfm/src/ymfm_pcm.cpp
    ./components/ymfm/src/ymfm_ssg.cpp
)

add_compile_flags(CXX
    -std=c++14
    -fno-exceptions
    -Wno-array-bounds
    -fPIC
#   -mbulk-memory
)

add_compile_flags(LD
)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ../dist)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ../dist)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ../dist)

add_executable(
    vgmrender.wasi
    ${SOURCE_FILES}
    ./src/cpp/vgmrender/em_inflate.cpp
    ./src/cpp/vgmrender/vgmrender.cpp
)

add_library(
    ymfm
    STATIC
    ${SOURCE_FILES}
    ./src/cpp/ffi/ymfmffi.cpp
)
