cmake_minimum_required(VERSION 2.8)
include(ExternalProject)

project(swallow)

add_subdirectory(swallow_client)

ExternalProject_Add(
  swallow
  CONFIGURE_COMMAND ""
  BUILD_COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}/swallow cargo build --release
  SOURCE_DIR "${CMAKE_SOURCE_DIR}/swallow"
  BINARY_DIR "${CMAKE_SOURCE_DIR}/swallow"
  BUILD_ALWAYS 1
  INSTALL_COMMAND ""
)

install(PROGRAMS ${CMAKE_CURRENT_BINARY_DIR}/swallow/release/swallow
  DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)
