fn main() {
    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=gcc");
    println!("cargo:rustc-link-search=/home/user/Documents/expirement/riscv32im-linux-x86_64/picolibc/riscv32-unknown-elf/lib");
    println!("cargo:rustc-link-search=/home/user/Documents/expirement/riscv32im-linux-x86_64/picolibc/lib/gcc/riscv32-unknown-elf/11.2.0");
    println!("cargo:rustc-link-arg=--allow-multiple-definition");
    //println!("cargo:rustc-link-arg=-T/home/user/Documents/expirement/risc0/risc0/build/risc0.ld");
    //risc0_build::link();
}
