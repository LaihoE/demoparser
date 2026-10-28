use std::{io::Result, path::PathBuf, process::Command};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=GameTracking-CS2/Protobufs/demo.proto");

    Command::new("git")
        .args([
            "clone",
            "https://github.com/SteamDatabase/GameTracking-CS2.git",
            "--depth=1",
        ])
        .status()?;

    let protos: Vec<PathBuf> = [
        "GameTracking-CS2/Protobufs/steammessages.proto",
        "GameTracking-CS2/Protobufs/gcsdk_gcmessages.proto",
        "GameTracking-CS2/Protobufs/demo.proto",
        "GameTracking-CS2/Protobufs/cstrike15_gcmessages.proto",
        "GameTracking-CS2/Protobufs/cstrike15_usermessages.proto",
        "GameTracking-CS2/Protobufs/usermessages.proto",
        "GameTracking-CS2/Protobufs/networkbasetypes.proto",
        "GameTracking-CS2/Protobufs/engine_gcmessages.proto",
        "GameTracking-CS2/Protobufs/netmessages.proto",
        "GameTracking-CS2/Protobufs/network_connection.proto",
        "GameTracking-CS2/Protobufs/cs_usercmd.proto",
        "GameTracking-CS2/Protobufs/usercmd.proto",
        "GameTracking-CS2/Protobufs/gameevents.proto",
        "GameTracking-CS2/Protobufs/cs_gameevents.proto",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect();

    // Compile with protox (a pure-Rust protobuf compiler) instead of protoc. Valve's
    // protos ship their own google/protobuf/descriptor.proto with custom options
    // (boxed_type, synthetic_default, additional_includes, ...) whose field numbers
    // collide with fields in the descriptor.proto that is compiled into protoc, so
    // recent protoc versions refuse to compile them. protox uses the descriptor.proto
    // from the include path, and it also means no protoc install is needed to build.
    let file_descriptor_set = protox::compile(&protos, ["GameTracking-CS2/Protobufs/"])
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    prost_build::Config::new()
        .format(false)
        .out_dir("src")
        .default_package_filename("protobuf")
        .bytes(["."])
        .enum_attribute(".", "#[derive(::strum::EnumIter)]")
        .skip_protoc_run()
        .compile_fds(file_descriptor_set)
}
