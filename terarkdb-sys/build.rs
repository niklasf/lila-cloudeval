use std::env;

fn main() {
    println!("cargo::rerun-if-changed=terarkdb");
    println!("cargo::rustc-link-search=native=terarkdb/output/lib");
    println!("cargo::rustc-link-lib=static=terarkdb");
}
