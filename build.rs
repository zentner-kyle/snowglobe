extern crate capnpc;

fn main() {
    let capnp_schema = [
        "schema/common.capnp",
        "schema/command.capnp",
        "schema/update.capnp"
    ];
    capnpc::compile("schema", &capnp_schema);
}
