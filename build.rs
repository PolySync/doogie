extern crate cmake;

use std::process::Command;

fn main() {
    // Update cmark submodule if needed
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .expect("Failed to run git command, is git installed?");

    // Build cmark
    let dst = cmake::build("cmark");

    // Inform Cargo where cmark artifacts are located
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("src").display());
    println!("cargo:rustc-link-lib=static=cmark");
}
