use protobuf_codegen::{Codegen, CustomizeCallback, Customize};
use protobuf::reflect::{EnumDescriptor};

fn main() {
Codegen::new()
    // Use `protoc` parser, optional.
    .protoc()
    // Use `protoc-bin-vendored` bundled protoc command, optional.
    //.protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
    // All inputs and imports from the inputs must reside in `includes` directories.
    .includes(&["src/protos/sources"])
    // Inputs must reside in some of include paths.
    .input("src/protos/sources/vega/events/v1/events.proto")
    .input("src/protos/sources/vega/chain_events.proto")
    .input("src/protos/sources/vega/vega.proto")
    .input("src/protos/sources/vega/assets.proto")
    .input("src/protos/sources/vega/markets.proto")
    .input("src/protos/sources/vega/oracles/v1/data.proto")
    .input("src/protos/sources/vega/oracles/v1/spec.proto")
    .input("src/protos/sources/vega/governance.proto")
    .input("src/protos/sources/vega/commands/v1/commands.proto")
    .input("src/protos/sources/vega/commands/v1/oracles.proto")
    .input("src/protos/sources/vega/commands/v1/signature.proto")
    .input("src/protos/sources/vega/commands/v1/validator_commands.proto")
    .customize_callback(DeriveFromToSql)
    // .input("src/protos/banana.proto")
    // Specify output directory relative to Cargo output directory.
    .cargo_out_dir("protos")
    .run_from_script();
}

struct DeriveFromToSql;

impl CustomizeCallback for DeriveFromToSql{ 
    fn enumeration(&self, _enum_type: &EnumDescriptor) -> Customize { 
        return Customize::default().before("#[derive(postgres_derive::FromSql, postgres_derive::ToSql)]
        #[postgres(name = \"transfer_type\")]
        ")
     }
}

