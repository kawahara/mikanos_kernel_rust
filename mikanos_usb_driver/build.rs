use std::{env, fs, os::unix, path::PathBuf};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn build_lib() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    // default variable for mikanos-docker
    let elf_dir = PathBuf::from(
        env::var("X86_64_ELF_DIR").unwrap_or("/home/vscode/osbook/devenv/x86_64-elf".to_string()),
    );
    let edk_dir = PathBuf::from(env::var("EDK2DIR").unwrap_or("/home/vscode/edk2".to_string()));

    env::set_var("CC", "clang");
    env::set_var("CXX", "clang++");

    let files = glob::glob("./cxx_src/**/*.cpp")?.collect::<std::result::Result<Vec<_>, _>>()?;

    cc::Build::new()
        .cpp(true)
        .include(elf_dir.join("include"))
        .include(elf_dir.join("include/c++/v1"))
        .include(edk_dir.join("MdePkg/Include"))
        .include(edk_dir.join("MdePkg/Include/X64"))
        .include("./cxx_src/")
        .files(files)
        .define("__ELF__", None)
        .define("_LDBL_EQ_DBL", None)
        .define("_GNU_SOURCE", None)
        .define("_POSIX_TIMERS", None)
        .flag("-nostdlibinc")
        .flag("-ffreestanding")
        .flag("-mno-red-zone")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-std=c++17")
        .extra_warnings(false)
        .cpp_link_stdlib(None)
        .target("x86_64-elf")
        .compile("mikanos_usb_driver");

    for lib in &["c", "c++", "c++abi"] {
        let filename = format!("lib{}.a", lib);
        let dest = out_dir.join(&filename);
        let src = elf_dir.join(format!("lib/{}", filename));
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        unix::fs::symlink(&src, &dest)?;
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    Ok(())
}

fn main() {
    build_lib().unwrap();
}
