use prometheus::{register_int_counter, register_histogram_with_opts, Histogram, HistogramOpts, IntCounter};
use once_cell::sync::Lazy;

pub static CEM_METRICS: Lazy<CemMetrics> = Lazy::new(|| CemMetrics::new());

pub struct CemMetrics {
    pub cem_calibrations_total: IntCounter,
    pub cem_calibration_mse: Histogram,
}

impl CemMetrics {
    pub fn new() -> Self {
        let cem_calibrations_total = register_int_counter!("cem_calibrations_total", "Total CEM calibrations").unwrap();
        let mse_opts = HistogramOpts::new("cem_calibration_mse", "CEM calibration MSE").buckets(vec![0.0001, 0.001, 0.01, 0.1, 1.0, 10.0]);
        let cem_calibration_mse = register_histogram_with_opts!(mse_opts).unwrap();
        Self { cem_calibrations_total, cem_calibration_mse }
    }
}
