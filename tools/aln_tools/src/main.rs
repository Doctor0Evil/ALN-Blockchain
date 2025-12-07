use clap::{Parser, Subcommand};
mod merkle;
use merkle::{build_merkle_and_proofs, Proof};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command(name = "aln-tools")]
pub struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SnapshotHash { input: String, output: Option<String>, asset_id: Option<String>, artifacts: Option<String> },
    Allocations { input: String, output: Option<String>, profile: Option<String>, c_e: Option<f64>, c_s: Option<f64>, d_src: u32, d_aln: u32 },
}

#[derive(Deserialize)]
struct SnapshotRow {
    address: String,
    denom: String,
    balance: String,
    height: u64,
    chain_id: String,
}

#[derive(Serialize)]
struct SnapshotHashOut {
    address: String,
    denom: String,
    balance: String,
    height: u64,
    chain_id: String,
    h_i: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::SnapshotHash { input, output, asset_id, artifacts } => snapshot_hash(&input, output.as_deref(), asset_id.as_deref(), artifacts.as_deref())?,
        Commands::Allocations { input, output, c_e, c_s, d_src, d_aln } => allocations(&input, output.as_deref(), c_e, c_s, d_src, d_aln)?,
    }
    Ok(())
}

fn snapshot_hash(path: &str, output: Option<&str>, asset_id: Option<&str>, artifacts: Option<&str>) -> anyhow::Result<()> {
    let data = std::fs::read_to_string(path)?;

    let rows: Vec<SnapshotRow> = if path.ends_with(".csv") {
        // read CSV
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let mut vec = vec![];
        for rec in rdr.deserialize() {
            vec.push(rec?);
        }
        vec
    } else {
        serde_json::from_str(&data)?
    };

    let mut out: Vec<SnapshotHashOut> = Vec::new();

    for r in rows {
        let mut hasher = Sha256::new();
        hasher.update(r.chain_id.as_bytes());
        hasher.update(&r.height.to_be_bytes());
        hasher.update(r.denom.as_bytes());
        hasher.update(r.address.as_bytes());
        let b: u128 = r.balance.parse()?;
        hasher.update(&b.to_be_bytes());
        let digest = hasher.finalize();
        let h_i = format!("0x{}", hex::encode(digest));

        out.push(SnapshotHashOut { address: r.address.clone(), denom: r.denom.clone(), balance: r.balance.clone(), height: r.height, chain_id: r.chain_id.clone(), h_i });
    }

    // sort by address, denom, balance for deterministic Merkle ordering
    out.sort_by(|a, b| {
        let a_key = format!("{}:{}:{}", a.address, a.denom, a.balance);
        let b_key = format!("{}:{}:{}", b.address, b.denom, b.balance);
        a_key.cmp(&b_key)
    });
    let json = serde_json::to_string_pretty(&out)?;
    if let Some(out_path) = output {
        let mut f = File::create(out_path)?;
        f.write_all(json.as_bytes())?;
    } else {
        println!("{}", json);
    }

    // If artifacts dir and asset id are provided via environment variables or additional flags, compute merkle root and proofs
    // Accept env var: ALN_ARTIFACTS_DIR and ALN_ASSET_ID
    // Write artifacts if specified via command-line flags (prefer flags to env vars)
    let artifacts_dir = artifacts.map(|s| s.to_string()).or_else(|| std::env::var("ALN_ARTIFACTS_DIR").ok());
    let asset_id = asset_id.map(|s| s.to_string()).or_else(|| std::env::var("ALN_ASSET_ID").ok());
    if let (Some(artifacts_dir), Some(asset_id)) = (artifacts_dir, asset_id) {
            // build leaves from out.h_i
            let mut leaves: Vec<[u8;32]> = vec![];
            for e in &out {
                let hex = e.h_i.trim_start_matches("0x");
                let bytes = hex::decode(hex)?;
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                leaves.push(arr);
            }
            let (root, proofs) = build_merkle_and_proofs(&leaves);
            // snapshot root artifact
            let snapshot_root = serde_json::json!({ "asset_id": asset_id.clone(), "merkle_root": root, "entries": out.iter().enumerate().map(|(i, e)| serde_json::json!({"index": i, "snapshot_hash": e.h_i })).collect::<Vec<_>>() });
            std::fs::create_dir_all(&artifacts_dir)?;
            let root_path = format!("{}/snapshot_root_{}.json", artifacts_dir, asset_id);
            let mut rf = File::create(root_path)?;
            rf.write_all(serde_json::to_string_pretty(&snapshot_root)?.as_bytes())?;
            // proofs
            let proofs_out: Vec<_> = out.iter().enumerate().map(|(i, e)| serde_json::json!({ "snapshot_hash": e.h_i, "proof": proofs[i].proof.iter().map(|ps| serde_json::json!({ "sibling": ps.sibling, "is_left": ps.is_left })).collect::<Vec<_>>() })).collect();
            let proof_path = format!("{}/merkle_proofs_{}.json", artifacts_dir, asset_id);
            let mut pf = File::create(proof_path)?;
            pf.write_all(serde_json::to_string_pretty(&proofs_out)?.as_bytes())?;
        }
    }

    // If artifacts and asset_id provided, write snapshot_root and merkle proofs
    // Use deterministic ordering by (address, denom, balance) before building merkle tree
    Ok(())

    Ok(())
}

fn allocations(path: &str, output: Option<&str>, profile: Option<String>, c_e: Option<f64>, c_s: Option<f64>, d_src: u32, d_aln: u32) -> anyhow::Result<()> {
    let data = std::fs::read_to_string(path)?;
    let rows: Vec<SnapshotRow> = if path.ends_with(".csv") {
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let mut vec = vec![];
        for rec in rdr.deserialize() {
            vec.push(rec?);
        }
        vec
    } else {
        serde_json::from_str(&data)?
    };

    #[derive(Serialize)]
    struct Out { address: String, auet: String, csp: String }

    let mut out_rows: Vec<Out> = Vec::new();

    // choose compression constants from profile or passed values
    let mut ce = c_e.unwrap_or(0.0);
    let mut cs = c_s.unwrap_or(0.0);
    if let Some(profile_id) = profile {
        // load config/scaling.yaml
        let cfg_str = std::fs::read_to_string("config/scaling.yaml")?;
        let cfg: serde_yaml::Value = serde_yaml::from_str(&cfg_str)?;
        if let Some(val) = cfg.get("profiles").and_then(|p| p.get(&profile_id)) {
            ce = val.get("c_e").and_then(|v| v.as_f64()).unwrap_or(ce);
            cs = val.get("c_s").and_then(|v| v.as_f64()).unwrap_or(cs);
        }
    }
    if ce == 0.0 || cs == 0.0 {
        if ce == 0.0 { ce = c_e.unwrap_or(1e-12); }
        if cs == 0.0 { cs = c_s.unwrap_or(5e-13); }
    }

    let factor_src = 10f64.powi(d_src as i32);
    let factor_aln = 10f64.powi(d_aln as i32);

    for r in rows {
        let b: f64 = r.balance.parse()?;
        let a_src = b / factor_src;
        let a_e = a_src * ce;
        let a_s = a_src * cs;

        let b_e = (a_e * factor_aln).floor() as u128;
        let b_s = (a_s * factor_aln).floor() as u128;

        out_rows.push(Out { address: r.address, auet: b_e.to_string(), csp: b_s.to_string() });
    }

    let json = serde_json::to_string_pretty(&out_rows)?;
    if let Some(out_path) = output {
        let mut f = File::create(out_path)?;
        f.write_all(json.as_bytes())?;
    } else {
        println!("{}", json);
    }

    Ok(())
}
