fn main() {
    tonic_build::compile_protos("proto/lmdb.proto").unwrap();
}
