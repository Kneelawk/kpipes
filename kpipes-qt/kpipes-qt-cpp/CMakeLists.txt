cmake_minimum_required(VERSION 3.10)

project(kpipes-qt)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

if (NOT RUST_LIB_LOCATION)
    set(RUST_LIB_LOCATION "../../target/x86_64-unknown-linux-gnu/debug")
endif()
message("Rust lib location: ${RUST_LIB_LOCATION}")

set(THREADS_PREFER_PTHREAD_FLAG ON)
find_package(Threads REQUIRED)

find_package(Qt5 COMPONENTS Widgets REQUIRED)

# specify the C++ standard
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED True)

add_library(kpipes-qt SHARED src/lib.cpp)

target_link_libraries(kpipes-qt PRIVATE Threads::Threads ${RUST_LIB_LOCATION}/libkpipes_qt_rust.a ${CMAKE_DL_LIBS} Qt5::Widgets)

target_include_directories(kpipes-qt PRIVATE ${RUST_LIB_LOCATION}/ffi)

install(TARGETS kpipes-qt)
