use anyhow::Result;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dir = if args.len() > 1 { &args[1] } else { "artifacts" };
    let max_bytes: u64 = if args.len() > 2 { args[2].parse().unwrap_or(2*1024*1024) } else { 2*1024*1024 };
    println!("Analyzing wasm files in {} (max {} bytes)", dir, max_bytes);
    fs::create_dir_all(dir)?;
    let mut exceeded = false;
    for ent in fs::read_dir(dir)? {
        let e = ent?;
        let p = e.path();
        if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext == "wasm" {
                    let meta = fs::metadata(&p)?;
                    let sz = meta.len();
                    println!("{}: {} bytes ({} KiB)", p.display(), sz, sz / 1024);
                    if sz > max_bytes { exceeded = true; println!("WARNING: {} exceeds max size", p.display()); }
                }
            }
        }
    }
    if exceeded { anyhow::bail!("WASM size threshold exceeded") }
    Ok(())
}
