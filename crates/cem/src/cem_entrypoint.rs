use crate::cem_calibration_workflow::{run_calibration, Sample, CalibrationParams};
use std::fs::File;
use std::io::{Read, Write};
use serde_json;
use clap::{Parser};
use std::path::PathBuf;

#[derive(Parser)]
pub struct CEMArgs {
    #[arg(long)]
    pub subject: u32,
    #[arg(long)]
    pub session: u64,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long, default_value_t = String::from("calibration/models"))]
    pub outdir: String,
    /// Optional metrics server address to enable Prometheus scraping (e.g., 127.0.0.1:9889)
    #[arg(long)]
    pub metrics_addr: Option<String>,
}

pub fn run_from_cli(args: &CEMArgs) -> anyhow::Result<CalibrationParams> {
    let mut f = File::open(&args.input)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let samples: Vec<Sample> = serde_json::from_str(&s)?;

    let params = run_calibration(&samples);

    let outdir = PathBuf::from(&args.outdir);
    std::fs::create_dir_all(&outdir)?;
    let outfile = outdir.join(format!("{}/params_v1.json", args.subject));
    let mut f = File::create(outfile)?;
    f.write_all(serde_json::to_string_pretty(&params)?.as_bytes())?;

    Ok(params)
}
