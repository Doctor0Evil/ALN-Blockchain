use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::Write;
use sha2::{Digest, Sha256};
use hex;

#[derive(Deserialize, Debug)]
struct DidConfig { controller_did: String }

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: did_provenance did-admin-check [--config config/did.json]");
        return Ok(());
    }
    let mut config_path = "config/did.json".to_string();
    if args.len() >= 3 { config_path = args[2].clone(); }
    let content = read_to_string(config_path)?;
    let cfg: DidConfig = serde_json::from_str(&content)?;
    let controller = cfg.controller_did;

    // operator DID from env var
    let operator = std::env::var("ALN_OPERATOR_DID").unwrap_or_default();
    if operator == "" {
        println!("ALN_OPERATOR_DID not set; consider setting or passing operator DID");
        std::process::exit(1);
    }
    if controller != operator {
        println!("Operator DID does not match controller DID");
        std::process::exit(1);
    }
    println!("DID admin check OK: {} == {}", controller, operator);
    // support subcommand: prove-wasm <wasm-path> <artifact-name>
    if args[1] == "prove-wasm" {
        if args.len() < 4 { println!("Usage: did_provenance prove-wasm <wasm-path> <artifact-name>"); std::process::exit(1); }
        let wasm_path = args[2].clone();
        let artifact_name = args[3].clone();
        let wasm_bin = std::fs::read(wasm_path.clone())?;
        let mut hasher = Sha256::new();
        hasher.update(&wasm_bin);
        let digest = hasher.finalize();
        let sha = format!("0x{}", hex::encode(digest));
        // optional optimized wasm path is artifact_name + ".optimized.wasm"
        let opt_path = format!("artifacts/{}.optimized.wasm", artifact_name);
        let mut opt_sha = None;
        if std::path::Path::new(&opt_path).exists() {
            let opt_bin = std::fs::read(&opt_path)?;
            let mut hasher = Sha256::new(); hasher.update(&opt_bin); let d = hasher.finalize(); opt_sha = Some(format!("0x{}", hex::encode(d)));
        }
        let git_sha = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "unknown".to_string());
        let run_id = std::env::var("GITHUB_RUN_ID").unwrap_or_else(|_| "local".to_string());
        #[derive(Serialize)]
        struct Provenance { did: String, wasm_sha: String, optimized_sha: Option<String>, commit: String, run_id: String }
        let prov = Provenance { did: cfg.controller_did.clone(), wasm_sha: sha, optimized_sha: opt_sha, commit: git_sha, run_id };
        let out_path = format!("artifacts/{}.provenance.json", artifact_name);
        std::fs::create_dir_all("artifacts")?;
        let mut f = File::create(out_path)?;
        f.write_all(serde_json::to_string_pretty(&prov)?.as_bytes())?;
        println!("Wrote provenance for {}", artifact_name);
        std::process::exit(0);
    }
    Ok(())
}
