use std::{io::Result, process::Command};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=../csgoproto/src/protobuf.rs");

    let profile = std::env::var("PROFILE").unwrap_or("debug".to_string());
    Command::new("cargo")
        .current_dir("../csgoproto")
        .args(["run", if profile == "release" { "--release" } else { "" }])
        .status()?;

    Ok(())
}
