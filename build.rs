extern crate capnpc;

fn main() {
    println!("running build.rs");
    let capnp_schema = [
        "schema/common.capnp",
        "schema/command.capnp",
        "schema/update.capnp"
    ];
    capnpc::compile("schema", &capnp_schema).unwrap();
}
