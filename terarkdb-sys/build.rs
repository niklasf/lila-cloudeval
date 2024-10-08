use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo::rerun-if-changed=terarkdb");
    println!(
        "cargo::rustc-link-search=native={}",
        manifest_dir
            .join("terarkdb")
            .join("output")
            .join("lib")
            .display()
    );
    println!("cargo::rustc-link-lib=static=terarkdb");
    println!("cargo::rustc-link-lib=static=terark-zip-r");
    println!("cargo::rustc-link-lib=static=boost_fiber");
    println!("cargo::rustc-link-lib=static=boost_context");
    println!("cargo::rustc-link-lib=static=gflags");
    println!("cargo::rustc-link-lib=static=bz2");
    println!("cargo::rustc-link-lib=static=lz4");
    println!("cargo::rustc-link-lib=static=snappy");
    println!("cargo::rustc-link-lib=static=z");

    println!("cargo::rustc-link-lib=dylib=tcmalloc");
    println!("cargo::rustc-link-lib=dylib=aio");
    println!("cargo::rustc-link-lib=dylib=gomp");
    println!("cargo::rustc-link-lib=dylib=stdc++");

    bindgen::builder()
        .layout_tests(false)
        .header("terarkdb/output/include/rocksdb/c.h")
        .allowlist_item("rocksdb_.*")
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();
}
