# A simple installer script for Rust and "generator" of headers & derived files.
#
# This must not try to replace Cargo and must not hold anything that can be in a
# simple `build.rs` build script. Anything Rust related must stay in Rust and
# Cargo.
#
# The goal for this is to help install the libraries (something that `cargo
# install` won't do) or create derived files like bindgens.

cmake_minimum_required(VERSION 3.10)

# Helpers

# Get a value in "keyword = value" from a line of Cargo.toml
function(cargo keyword out)
    # These should be out, but make for a cleaner function this way
    file(READ "Cargo.toml" raw_cargo)
    string(REPLACE "\n" ";" cargo "${raw_cargo}")

    foreach(line IN LISTS cargo)
        set(pre "${line}")
        string(REGEX REPLACE
            "${keyword}[\t ]*=[\t ]*\"(.*)\"" "\\1" post "${line}")

        # replacement was successful so the line is found
        if(NOT (pre STREQUAL post))
            set(${out} "${post}" PARENT_SCOPE)
            return()
        endif()
    endforeach()

    message(
      WARNING
      "search for the key \"${keyword} = VALUE\" in Cargo.toml file has failed"
    )
endfunction()

# set the values from Cargo
cargo("name" name)
cargo("version" version)
cargo("description" description)
cargo("repository" homepage)

# Essentials

project("${name}"
    VERSION "${version}"
    DESCRIPTION "${description}"
    HOMEPAGE_URL "${homepage}"
    LANGUAGES NONE # no way to compile officially yet
)

# Constants

if("${CMAKE_BUILD_TYPE}" STREQUAL "")
    set(CMAKE_BUILD_TYPE "Debug")
endif()

string(TOLOWER "${CMAKE_BUILD_TYPE}" mode)
set(release_flag "$<$<STREQUAL:release,${mode}>:--release>")
set(src "${PROJECT_SOURCE_DIR}") # working directory
set(target "${src}/target")
set(c_header "${target}/c_header.h")

execute_process(COMMAND rustup default OUTPUT_VARIABLE default)
string(REPLACE "-" ";" default "${default}")
list(GET default 0 default)

# Targets

add_custom_target(Lib ALL
    cargo build "${release_flag}" --lib
    BYPRODUCTS "${target}/${mode}/"
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)

add_custom_target(Doc
    cargo doc "${release_flag}"
    BYPRODUCTS "${target}/doc/"
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)

# since macro expansion is used for cbindgen, set to nightly and back
add_custom_target(Cbindgen
    cargo build "${release_flag}" --lib --features=c
    COMMAND rustup default nightly
    COMMAND cbindgen --crate "${name}" --output "${c_header}" --lang c
    COMMAND rustup default "${default}"
    BYPRODUCTS "${c_header}"
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)

add_custom_target(Wasm32
    cargo build "${release_flag}" --lib --target=wasm32-unknown-unknown --features=wasm
    BYPRODUCTS "${target}/wasm32-unknown-unknown/"
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)

add_custom_target(WasmPack
    wasm-pack build "${release_flag}" --out-dir "${target}/pkg" --features=wasm
    BYPRODUCTS "${target}/pkg/"
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)
add_dependencies(WasmPack Wasm32)

add_custom_target(Maturin
    maturin build "${release_flag}" --features=py
    WORKING_DIRECTORY "${src}"
    VERBATIM
    COMMAND_EXPAND_LISTS)

# Everything is not `ALL` as this CMake is intended to keep that with the
# minimal dependencies as to make it not to halt if tools are not prepare for
# anybody who invokes cmake without any explicit target
add_custom_target(Everything)
add_dependencies(Everything Lib Doc Cbindgen Wasm32 WasmPack Maturin)

# Install

install(FILES
    "$<$<PLATFORM_ID:Linux>:${target}/${mode}/lib${name}.so>"
    "$<$<PLATFORM_ID:Windows>:${target}/${mode}/lib${name}.dll>"
    "$<$<PLATFORM_ID:Darwin>:${target}/${mode}/lib${name}.dylib>"
    TYPE LIB) # for adding Windows support in an easy hacky way

install(FILES
    "${target}/${mode}/lib${name}.a"
    "$<$<PLATFORM_ID:Windows>:${target}/${mode}/lib${name}.lib>"
    TYPE LIB
    OPTIONAL)

install(FILES "${c_header}"
    TYPE INCLUDE
    RENAME "${name}.h"
    OPTIONAL)

# TODO add install for js
# TODO add install for Mac
# TODO clean up
# TODO add for creating an env and develop of maturin
