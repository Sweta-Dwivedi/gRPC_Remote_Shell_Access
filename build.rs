use tonic_build;

fn main() {
    tonic_build::compile_protos("proto/remote_shell.proto").unwrap();
}
