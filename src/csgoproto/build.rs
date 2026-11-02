use std::{io::Result, process::Command};

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=GameTracking-CS2/Protobufs/demo.proto");

    Command::new("git")
        .args([
            "clone",
            "https://github.com/SteamDatabase/GameTracking-CS2.git",
            "--depth=1",
        ])
        .status()?;

    let protos = vec![
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
    ];

    prost_build::Config::new()
        .format(false)
        .out_dir("src")
        .default_package_filename("protobuf")
        .bytes(["."])
        .enum_attribute(".", "#[derive(::strum::EnumIter)]")
        .compile_protos(&protos, &["GameTracking-CS2/Protobufs/"])?;

    // The published protos lack tag 6 (`delta_data`) on CMsgServerUserCmd,
    // which the parser reads for delta-encoded usercmds. Re-apply it after
    // every codegen run (idempotent); without it fresh builds don't compile.
    let generated = std::fs::read_to_string("src/protobuf.rs")?;
    if !generated.contains("pub delta_data") {
        let needle = "    pub client_tick: ::core::option::Option<i32>,\n}\n#[derive(Clone, PartialEq, ::prost::Message)]\npub struct CsvcMsgUserCommands {";
        let replacement = "    pub client_tick: ::core::option::Option<i32>,\n    #[prost(bytes=\"bytes\", optional, tag=\"6\")]\n    pub delta_data: ::core::option::Option<::prost::bytes::Bytes>,\n}\n#[derive(Clone, PartialEq, ::prost::Message)]\npub struct CsvcMsgUserCommands {";
        match generated.replacen(needle, replacement, 1) {
            patched if patched != generated => std::fs::write("src/protobuf.rs", patched)?,
            _ => panic!("csgoproto build: CMsgServerUserCmd anchor not found in regenerated protobuf.rs"),
        }
    }
    Ok(())
}
