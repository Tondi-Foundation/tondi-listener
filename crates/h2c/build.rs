use tonic_prost_build::{Config, configure};

const RKYV_CODEC: &str = "crate::codec::rkyv::Codec";
const RKYV_ATTR: &str = "#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]";

#[derive(Debug)]
struct Compiler {}

impl Compiler {
    fn compile(dir: &str, protos: &[&str]) -> std::io::Result<()> {
        let mut config = Config::new();
        let mut builder = configure().out_dir("target/proto");
        
        if cfg!(feature = "rkyv-codec") {
            config.type_attribute(".", RKYV_ATTR);
            builder = builder.codec_path(RKYV_CODEC);
        }

        let protos: Vec<String> = protos.iter().map(|name| format!("{dir}/{name}.proto")).collect();
        builder.compile_with_config(config, &protos, &[dir.into()])
    }
}

fn main() -> std::io::Result<()> {
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all("target/proto")?;
    
    // Get the workspace root directory (go up two levels from crates/h2c)
    let current_dir = std::env::current_dir()?;
    let workspace_root = current_dir
        .parent()
        .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find parent directory"))?
        .parent()
        .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find workspace root"))?;
    std::env::set_current_dir(&workspace_root)?;
    
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all("target/proto")?;
    
    // Pingpong
    Compiler::compile("protowire", &["pingpong"])?;
    // Explorer
    Compiler::compile("protowire/explorer", &["lib", "transaction", "block", "service"])?;
    Ok(())
}
