include(CheckFunctionExists)
include(CheckSymbolExists)
include(CheckIncludeFile)
include(GNUInstallDirs)
include(CheckCCompilerFlag)

check_c_compiler_flag("-std=c99" C99_SUPPORTED)
if (C99_SUPPORTED)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -std=c99")
endif()

add_definitions(-D_GNU_SOURCE)
list(APPEND CMAKE_REQUIRED_DEFINITIONS -D_GNU_SOURCE)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Release")
endif()

set(CMAKE_OSX_ARCHITECTURES "i386;x86_64" CACHE STRING  "Rogue")

check_function_exists(execve HAVE_EXECVE)
check_function_exists(execv HAVE_EXECV)
check_function_exists(execvpe HAVE_EXECVPE)
check_function_exists(execvp HAVE_EXECVP)
check_function_exists(execvP HAVE_EXECVP2)
check_function_exists(exect HAVE_EXECT)
check_function_exists(execl HAVE_EXECL)
check_function_exists(execlp HAVE_EXECLP)
check_function_exists(execle HAVE_EXECLE)
check_function_exists(posix_spawn HAVE_POSIX_SPAWN)
check_function_exists(posix_spawnp HAVE_POSIX_SPAWNP)
check_symbol_exists(_NSGetEnviron crt_externs.h HAVE_NSGETENVIRON)
check_include_file(xlocale.h HAVE_XLOCALE_HEADER)

find_package(Threads REQUIRED)

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/c/config.h.in ${CMAKE_CURRENT_BINARY_DIR}/config.h)
include_directories(${CMAKE_CURRENT_BINARY_DIR})

add_library(swallow_client SHARED c/swallow_client.c)
target_link_libraries(swallow_client ${CMAKE_DL_LIBS} ${CMAKE_CURRENT_BINARY_DIR}/swallow_client_rust/release/libswallow_client_rust.a)
if(THREADS_HAVE_PTHREAD_ARG)
    set_property(TARGET swallow_client PROPERTY COMPILE_OPTIONS "-pthread")
    set_property(TARGET swallow_client PROPERTY INTERFACE_COMPILE_OPTIONS "-pthread")
endif()
if(CMAKE_THREAD_LIBS_INIT)
    target_link_libraries(swallow_client "${CMAKE_THREAD_LIBS_INIT}")
endif()

if(APPLE)
    set(CMAKE_MACOSX_RPATH 1)
    set_target_properties(swallow_client PROPERTIES INSTALL_RPATH "@loader_path/${EAR_LIB_PATH}")
endif()

install(TARGETS swallow_client
    LIBRARY DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)

ExternalProject_Add(
  swallow_client_rust
  CONFIGURE_COMMAND ""
  BUILD_COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}/swallow_client_rust cargo build --release
  SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/rust"
  BINARY_DIR "${CMAKE_CURRENT_SOURCE_DIR}/rust"
  BUILD_ALWAYS 1
  INSTALL_COMMAND ""
)
add_dependencies(swallow_client swallow_client_rust)


