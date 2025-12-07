use clap::Parser;
use cem::cem_entrypoint::{CEMArgs, run_from_cli};
use tracing_subscriber;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().flatten_event(true).with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    // start simple metrics server for CEM
    // Metrics opt-in via CLI flag or environment variable. If neither are provided we skip starting the /metrics server.
    let args = CEMArgs::parse();
    let metrics_addr = args.metrics_addr.clone().or_else(|| std::env::var("CEM_METRICS_ADDR").ok());
    if let Some(addr) = metrics_addr {
        let addr_clone = addr.clone();
        let metrics_route = warp::path("metrics").and_then(|| async move { let s = prometheus::gather(); let encoder = prometheus::TextEncoder::new(); let mut buffer = Vec::new(); encoder.encode(&s, &mut buffer).unwrap(); Ok::<_, std::convert::Infallible>(warp::reply::with_header(String::from_utf8(buffer).unwrap(), "content-type", "text/plain; version=0.0.4")) });
        tokio::spawn(async move { warp::serve(metrics_route).run(addr_clone.parse().unwrap()).await; });
    }
    // args already parsed above to check metrics; continue to use them.
    // run_from_cli is synchronous – it returns a result
    let params = run_from_cli(&args)?;
    println!("Calibration params: {:#?}", params);
    Ok(())
}
