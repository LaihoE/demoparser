fn main() {
    protobuf_codegen::Codegen::new()
        .protoc()
        .includes(&["Protobufs/csgo/"])
        .input("Protobufs/csgo/demo.proto")
        .input("Protobufs/csgo/cstrike15_gcmessages.proto")
        .input("Protobufs/csgo/usermessages.proto")
        .input("Protobufs/csgo/networkbasetypes.proto")
        .input("Protobufs/csgo/engine_gcmessages.proto")
        .input("Protobufs/csgo/steammessages.proto")
        .input("Protobufs/csgo/netmessages.proto")
        .input("Protobufs/csgo/network_connection.proto")
        .input("Protobufs/csgo/gcsdk_gcmessages.proto")
        .out_dir("src2/")
        .customize(protobuf_codegen::Customize::default().tokio_bytes(true))
        .run_from_script();
}
