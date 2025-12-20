fn main() {
    prost_build::compile_protos(&["proto/kv.proto"], &["proto/"]).unwrap();
}
