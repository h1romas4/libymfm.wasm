cmake_minimum_required(VERSION 3.0)

set(CMAKE_SYSTEM_NAME WASI)
set(CMAKE_SYSTEM_VERSION 1)
set(CMAKE_SYSTEM_PROCESSOR wasm32)
set(CMAKE_SYSROOT $ENV{WASI_SDK_PATH}/share/wasi-sysroot/)
set(triple wasm32-wasi)

if(WIN32)
	set(WASI_HOST_EXE_SUFFIX ".exe")
else()
	set(WASI_HOST_EXE_SUFFIX "")
endif()

set(CMAKE_C_COMPILER           $ENV{WASI_SDK_PATH}/bin/clang${WASI_HOST_EXE_SUFFIX})
set(CMAKE_CXX_COMPILER         $ENV{WASI_SDK_PATH}/bin/clang++${WASI_HOST_EXE_SUFFIX})
set(CMAKE_AR                   $ENV{WASI_SDK_PATH}/bin/llvm-ar${WASI_HOST_EXE_SUFFIX})
set(CMAKE_RANLIB               $ENV{WASI_SDK_PATH}/bin/llvm-ranlib${WASI_HOST_EXE_SUFFIX})
set(CMAKE_C_COMPILER_TARGET    ${triple})
set(CMAKE_CXX_COMPILER_TARGET  ${triple})

include(./cmake/macros.cmake)
project(libymfm.wasi C CXX)

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
    ./src/em_inflate.cpp
    ./src/vgmrender.cpp
    ./src/wasmymfm.cpp
)

add_compile_flags(CXX
    -std=c++14
    -fno-exceptions
    -Wno-array-bounds
)

add_compile_flags(LD
)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ../dist)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ../dist)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ../dist)

add_executable(${PROJECT_NAME} ${SOURCE_FILES})
