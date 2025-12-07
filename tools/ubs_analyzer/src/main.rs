use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct UbsReport {
    denom: String,
    issues: Vec<String>,
    severity: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: ubs_analyzer <denom> <file>\n    or: ubs_analyzer print-hash <file>\n");
        return Ok(());
    }
    if args[1] == "print-hash" {
        if args.len() < 3 { println!("Usage: ubs_analyzer print-hash <file>"); return Ok(()); }
        let path = args[2].clone();
        let content = std::fs::read_to_string(path)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(content.as_bytes());
        let digest = hasher.finalize();
        println!("0x{}", hex::encode(digest));
        return Ok(());
    }
    let denom = args[1].clone();
    let path = args[2].clone();
    let content = read_to_string(path)?;

    // naive checks for common risky patterns
    let mut issues: Vec<String> = Vec::new();
    let mut sev = 0;

    let mint_re = Regex::new(r"mint\(|minted|_mint|minting")?;
    if mint_re.is_match(&content) {
        issues.push("mint authority or calls detected".to_string());
        sev += 3;
    }
    let admin_re = Regex::new(r"admin|governor|owner|multisig|accesscontrol|grantRole")?;
    if admin_re.is_match(&content) {
        issues.push("admin or role-based operations detected".to_string());
        sev += 2;
    }
    let freeze_re = Regex::new(r"freeze|blacklist|pause|seize|drain")?;
    if freeze_re.is_match(&content) {
        issues.push("freeze/blacklist/seize functions detected".to_string());
        sev += 3;
    }

    let severity = if sev >= 6 { "high" } else if sev >= 3 { "medium" } else { "low" };
    let report = UbsReport { denom: denom.clone(), issues, severity: severity.to_string() };

    let json = serde_json::to_string_pretty(&report)?;
    let out_file = format!("artifacts/ubs_report_{}.json", denom.replace('/', "_"));
    let mut f = File::create(out_file)?;
    f.write_all(json.as_bytes())?;
    println!("Wrote UBS report for {}", denom);

    Ok(())
}
