use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Deserialize)]
struct DenomTrace {
    pub path: String,
    pub base_denom: String,
}

#[derive(Debug, Deserialize)]
struct DenomTracesResponse {
    pub denom_traces: Vec<DenomTrace>,
    pub pagination: Value,
}

#[derive(Debug, Serialize)]
struct OrphanRecord {
    pub ibc_denom: String,
    pub path: String,
    pub base_denom: String,
    pub reason: String,
}

fn main() -> anyhow::Result<()> {
    // Config: known base denoms
    let known_bases: HashSet<String> = vec![
        "uatom".to_string(),
        "uosmo".to_string(),
        "axlUSDC".to_string(),
        "ukuji".to_string(),
    ]
    .into_iter()
    .collect();

    let mut orphan_records: Vec<OrphanRecord> = Vec::new();
    let mut next_key = String::new();

    loop {
        let url = if next_key.is_empty() {
            "https://kujira-api.polkachu.com/ibc/apps/transfer/v1/denom_traces?pagination.limit=1000".to_string()
        } else {
            format!(
                "https://kujira-api.polkachu.com/ibc/apps/transfer/v1/denom_traces?pagination.key={}",
                urlencoding::encode(&next_key)
            )
        };

        let resp: DenomTracesResponse = reqwest::blocking::get(&url)?.json()?;
        for trace in &resp.denom_traces {
            let full = if trace.path.is_empty() {
                trace.base_denom.clone()
            } else {
                format!("{}/{}", trace.path, trace.base_denom)
            };

            let hash = Sha256::digest(full.as_bytes());
            let ibc_denom = format!("ibc/{}", hex::encode(hash));

            if !known_bases.contains(&trace.base_denom) {
                orphan_records.push(OrphanRecord {
                    ibc_denom,
                    path: trace.path.clone(),
                    base_denom: trace.base_denom.clone(),
                    reason: "base_denom_not_in_registry".to_string(),
                });
            }
        }

        // Break out of the loop if no next key or pagination end
        if let Some(next) = resp.pagination.get("next_key") {
            if next.is_null() {
                break;
            }
            if let Some(k) = next.as_str() {
                next_key = k.to_string();
                if next_key.is_empty() {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Write orphan set to file
    let json = serde_json::to_string_pretty(&orphan_records)?;
    let mut f = File::create("artifacts/orphan_ibc.json")?;
    f.write_all(json.as_bytes())?;
    println!("Wrote artifacts/orphan_ibc.json ({} orphans)", orphan_records.len());

    Ok(())
}
