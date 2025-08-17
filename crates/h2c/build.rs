use tonic_prost_build::{Config, configure};

const PROTO_DIR: &str = env!("PROTOWIRE_DIR");
const PROTO_GEN: &str = env!("PROTO_GEN_DIR");

const RKYV_CODEC: &str = "crate::codec::rkyv::Codec";
const RKYV_ATTR: &str = "#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]";

#[derive(Debug)]
struct Compiler {}

impl Compiler {
    fn compile(dir: &str, protos: &[&str]) -> std::io::Result<()> {
        let mut config = Config::new();
        let mut builder = configure().out_dir(PROTO_GEN);
        if cfg!(feature = "rkyv-codec") {
            config.type_attribute(".", RKYV_ATTR);
            builder = builder.codec_path(RKYV_CODEC);
        }

        let protos: Vec<String> = protos.iter().map(|name| format!("{dir}/{name}.proto")).collect();
        builder.compile_with_config(config, &protos, &[dir.into()])
    }
}

fn main() -> std::io::Result<()> {
    // Pingpong
    Compiler::compile(PROTO_DIR, &["pingpong"])?;
    // Explorer
    let explorer = format!("{PROTO_DIR}/explorer");
    Compiler::compile(&explorer, &["lib", "transaction", "block", "service"])?;
    Ok(())
}
