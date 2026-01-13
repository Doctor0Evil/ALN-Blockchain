use metrics::{describe_counter, increment_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use once_cell::sync::Lazy;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum MetricsError {
    Start(std::io::Error),
}

static EXPORTER_STARTED: Lazy<()> = Lazy::new(|| {});

pub fn init_metrics_exporter(addr: SocketAddr) -> Result<(), MetricsError> {
    Lazy::force(&EXPORTER_STARTED);
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()
        .map_err(MetricsError::Start)?;
    describe_counter!("bciduty_cycle", "BCI duty cycle usage.");
    describe_counter!("aln_noncompliant", "ALN noncompliant events.");
    Ok(())
}

pub fn record_bci_duty_cycle() {
    increment_counter!("bciduty_cycle");
}

pub fn record_aln_noncompliant() {
    increment_counter!("aln_noncompliant");
}
