use std::io::Result;

fn main() -> Result<()> {
    let protos = vec![
        "Protobufs/csgo/steammessages.proto",
        "Protobufs/csgo/gcsdk_gcmessages.proto",
        "Protobufs/csgo/demo.proto",
        "Protobufs/csgo/cstrike15_gcmessages.proto",
        "Protobufs/csgo/cstrike15_usermessages.proto",
        "Protobufs/csgo/usermessages.proto",
        "Protobufs/csgo/networkbasetypes.proto",
        "Protobufs/csgo/engine_gcmessages.proto",
        "Protobufs/csgo/netmessages.proto",
        "Protobufs/csgo/network_connection.proto",
        "Protobufs/csgo/cs_usercmd.proto",
        "Protobufs/csgo/usercmd.proto",
    ];

    prost_build::Config::new()
        .format(false)
        .out_dir("src")
        .default_package_filename("lib")
        .bytes(["."])
        .compile_protos(&protos, &["Protobufs/csgo/"])?;

    Ok(())
}