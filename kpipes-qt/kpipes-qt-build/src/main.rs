use clap::{App, Arg};
use std::{fs::canonicalize, path::Path};
use std::process::Command;

macro_rules! run_command {
    ($command:expr) => {{
        let status = $command.status().unwrap();
        if !status.success() {
            panic!("Command {:?} returned status: {}", $command, status);
        }
    }};
}

macro_rules! mkdir {
    ($dir:expr) => {
        if !$dir.exists() {
            std::fs::create_dir($dir).unwrap();
        }
    };
}

const CPP_PATH: &str = "kpipes-qt/kpipes-qt-cpp";
const RUST_PACKAGE: &str = "kpipes-qt-rust";

const CMAKE_DIR: &str = "cmake-build";

fn main() {
    let matches = App::new("rust-cpp builder")
        .arg(Arg::with_name("target").long("target").takes_value(true))
        .arg(Arg::with_name("profile").long("profile").takes_value(true))
        .get_matches();

    let target = matches
        .value_of("target")
        .expect("Missing --target argument");
    let profile = matches
        .value_of("profile")
        .expect("Missing --profile argument")
        .to_lowercase();

    let host = guess_host_triple::guess_host_triple().expect("Unable to guess host triple");

    let cpp_dir = canonicalize(Path::new(CPP_PATH)).expect("error getting cpp-path");

    let mut cargo = Command::new("cargo");
    cargo.args(&["build", "--target", target, "--package", RUST_PACKAGE]);
    if profile == "release" {
        cargo.arg("--release");
    }
    run_command!(cargo);

    let build_dir = canonicalize(Path::new("target").join(target).join(&profile))
        .expect("error getting build dir path");

    let cmake_dir = build_dir.join(CMAKE_DIR);
    mkdir!(&cmake_dir);
    let mut cmake = cmake::Config::new(&cpp_dir);
    cmake.out_dir(&cmake_dir);
    cmake.target(target);
    cmake.host(host);
    cmake.profile(&profile.to_uppercase());
    cmake.define("RUST_LIB_LOCATION", build_dir.to_str().unwrap());
    cmake.build();
}
