use std::{io::Result, process::Command};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=../csgoproto/src/protobuf.rs");
    println!("cargo::rerun-if-changed=../csgoproto/GameTracking-CS2/game/csgo/pak01_dir/resource/csgo_english.txt");

    let profile = std::env::var("PROFILE").unwrap_or("debug".to_string());
    Command::new("cargo")
        .current_dir("../csgoproto")
        .args(["run", if profile == "release" { "--release" } else { "" }])
        .status()?;

    Ok(())
}
