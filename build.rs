fn main() {
    match prost_build::compile_protos(&["proto/messages.proto"], &["proto/"]) {
        Ok(_) => println!("cargo:rerun-if-changed=proto/messages.proto"),
        Err(e) => panic!("Failed to compile protos {:?}", e),
    }
}
