use chrono::Utc;
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "target/manifests")]
    emit: String,
}

#[derive(Debug, Serialize)]
struct EvolutionManifest {
    created_at: String,
    version: String,
}

fn main() {
    let args = Args::parse();
    let dir = PathBuf::from(&args.emit);
    fs::create_dir_all(&dir).expect("create manifest dir");
    let manifest = EvolutionManifest {
        created_at: Utc::now().to_rfc3339(),
        version: "0.1.0".into(),
    };
    let filename = format!("research-{}-manifest.json", Utc::now().format("%Y%m%d"));
    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(&manifest).expect("serialize");
    fs::write(path, json).expect("write manifest");
}
