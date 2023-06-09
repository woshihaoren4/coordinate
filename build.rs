fn main() {
    tonic_build::configure()
        .out_dir("./src/proto/")
        .file_descriptor_set_path("./src/proto/services_descriptor.bin")
        .build_client(true)
        .build_server(true)
        // .type_attribute("hello.HelloResponse","#[derive(serde::Serialize,serde::Deserialize)]")
        // .type_attribute("hello.HelloRequest","#[derive(serde::Serialize,serde::Deserialize)]")
        .emit_rerun_if_changed(true)
        .compile(
            &[
                "./proto/message.proto",
                "./proto/task_service.proto",
                "./proto/node_service.proto",
            ],
            &["proto"],
        )
        .unwrap();

    // tonic_build::configure()
    //     .out_dir("./src/proto")
    //     .file_descriptor_set_path("./src/proto/hello_descriptor.bin")
    //     // .type_attribute("hello.HelloResponse","#[derive(serde::Serialize,serde::Deserialize)]")
    //     // .type_attribute("hello.HelloRequest","#[derive(serde::Serialize,serde::Deserialize)]")
    //     .compile(&["./proto/hello.proto"],&["proto"])
    //     .unwrap();
}
