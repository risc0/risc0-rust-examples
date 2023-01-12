use risc0_zkvm::build::rcc::Build;
use std::{
    env,
    path::{Path, PathBuf},
};

use glob::glob;

fn main() {
    // TODO: change location of riscv32im toolchain e.g. extracted from tarball available at
    // https://github.com/risc0/toolchain/releases/
    let RV32IM_TOOLS_PATH = Path::new("/home/user/Documents/expirement/riscv32im-linux-x86_64");

    let LIBC_SUFFIX = Path::new("picolibc/riscv32-unknown-elf/lib/libc.a");
    let LIBC_LOC = Path::join(RV32IM_TOOLS_PATH, LIBC_SUFFIX);
    let LIBC_LOC_STR = LIBC_LOC.to_str().unwrap();
    println!("cargo:rustc-link-lib={LIBC_LOC_STR}");
    //println!(concat!("cargo:rustc-link-arg=-T/home/user/Documents/expirement/risc0/risc0/build/risc0.ld");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let inc_dir = Path::new(&manifest_dir);
    println!("cargo:include={}", inc_dir.to_str().unwrap());
    let srcs: Vec<PathBuf> = glob("chibicc/*.c")
        .unwrap()
        .map(|x| x.unwrap())
        .collect();

    let mut build = Build::new();

    let COMPILER_SUFFIX = Path::new("bin/riscv32-unknown-elf-gcc");
    let COMPILER_LOC = Path::join(RV32IM_TOOLS_PATH, COMPILER_SUFFIX);
    let COMPILER_LOC_STR = COMPILER_LOC.to_str().unwrap();

    let LIBC_INCLUDE_SUFFIX = Path::new("picolibc/include");
    let LIBC_INCLUDE_LOC = Path::join(RV32IM_TOOLS_PATH, LIBC_INCLUDE_SUFFIX);
    let LIBC_INCLUDE_STR = LIBC_INCLUDE_LOC.to_str().unwrap();
    build
        .compiler(COMPILER_LOC_STR)
        .files(srcs)
		.file("chibicc/chibicc.h")
        .include(LIBC_INCLUDE_STR)
        .warnings(false);

    build.compile("chibicc");
}
