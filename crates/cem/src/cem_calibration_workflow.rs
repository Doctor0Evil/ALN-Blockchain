use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub timestamp_ns: u64,
    pub subject_id: u32,
    pub session_id: u64,
    pub segment_id: String,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub gx: f32,
    pub gy: f32,
    pub gz: f32,
    pub mx: f32,
    pub my: f32,
    pub mz: f32,
    pub f_normal: f32,
    pub f_tangential: f32,
    pub event_count: f32,
    pub event_polarity_mean: f32,
    pub eeg_band_power: Vec<f32>,
    pub emg_rms: f32,
    pub p_mw_measured: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationParams {
    pub alpha_v2: f64,
    pub beta_a2: f64,
    pub gamma_neural: f64,
    pub delta_force: f64,
    pub intercept: f64,
    pub mse: f64,
    pub hash_hex: String,
}

pub fn sanitize(samples: &[Sample]) -> Vec<Sample> {
    let clamp = |x: f32, lo: f32, hi: f32| x.max(lo).min(hi);
    samples
        .iter()
        .cloned()
        .map(|mut s| {
            s.ax = clamp(s.ax, -200.0, 200.0);
            s.ay = clamp(s.ay, -200.0, 200.0);
            s.az = clamp(s.az, -200.0, 200.0);
            s.gx = clamp(s.gx, -2_000.0, 2_000.0);
            s.gy = clamp(s.gy, -2_000.0, 2_000.0);
            s.gz = clamp(s.gz, -2_000.0, 2_000.0);
            s.f_normal = clamp(s.f_normal, 0.0, 5_000.0);
            s.f_tangential = clamp(s.f_tangential, -5_000.0, 5_000.0);
            s.event_count = clamp(s.event_count, 0.0, 10_000.0);
            s.event_polarity_mean = clamp(s.event_polarity_mean, -1.0, 1.0);
            s.emg_rms = clamp(s.emg_rms, 0.0, 10.0);
            s.p_mw_measured = clamp(s.p_mw_measured, 0.0, 10_000.0);
            s
        })
        .collect()
}

fn extract_features(samples: &[Sample]) -> (Vec<[f64; 5]>, Vec<f64>) {
    let mut x_rows = Vec::with_capacity(samples.len());
    let mut y_vals = Vec::with_capacity(samples.len());

    for s in samples {
        let v_norm2 = (s.gx as f64).powi(2) + (s.gy as f64).powi(2) + (s.gz as f64).powi(2);
        let a_norm2 = (s.ax as f64).powi(2) + (s.ay as f64).powi(2) + (s.az as f64).powi(2);
        let neural_scalar: f64 = if !s.eeg_band_power.is_empty() {
            s.eeg_band_power.iter().map(|v| *v as f64).sum::<f64>() / (s.eeg_band_power.len() as f64)
        } else {
            s.emg_rms as f64
        };
        let force_scalar = (s.f_normal.powi(2) + s.f_tangential.powi(2)).sqrt() as f64;
        let p = s.p_mw_measured as f64;

        x_rows.push([v_norm2, a_norm2, neural_scalar, force_scalar, 1.0]);
        y_vals.push(p);
    }

    (x_rows, y_vals)
}

#[tracing::instrument(skip(x_rows, y_vals))]
pub fn fit_params(x_rows: &[[f64; 5]], y_vals: &[f64]) -> CalibrationParams {
    assert!(x_rows.len() >= 5, "Need at least 5 samples");

    let n = x_rows.len();
    let d = 5;
    let mut xtx = vec![0.0f64; d * d];
    let mut xty = vec![0.0f64; d];

    for i in 0..n {
        let row = &x_rows[i];
        let y = y_vals[i];
        for r in 0..d {
            xty[r] += row[r] * y;
            for c in 0..d {
                xtx[r * d + c] += row[r] * row[c];
            }
        }
    }

    let theta = gaussian_solve_5x5(&xtx, &xty);

    let alpha_v2 = theta[0];
    let beta_a2 = theta[1];
    let gamma_neural = theta[2];
    let delta_force = theta[3];
    let intercept = theta[4];

    let mut mse = 0.0;
    for i in 0..n {
        let row = &x_rows[i];
        let y = y_vals[i];
        let y_hat = alpha_v2 * row[0] + beta_a2 * row[1] + gamma_neural * row[2] + delta_force * row[3] + intercept * row[4];
        let e = y - y_hat;
        mse += e * e;
    }
    mse /= n as f64;

    let mut hasher = Sha256::new();
    for v in [alpha_v2, beta_a2, gamma_neural, delta_force, intercept, mse] {
        hasher.update(v.to_le_bytes());
    }
    let hash_hex = format!("{:x}", hasher.finalize());

    let params = CalibrationParams { alpha_v2, beta_a2, gamma_neural, delta_force, intercept, mse, hash_hex };
    tracing::info!(mse = params.mse, alpha = params.alpha_v2, beta = params.beta_a2, "calibration fit complete");
    // Update CEM metrics
    crate::metrics::CEM_METRICS.cem_calibrations_total.inc();
    crate::metrics::CEM_METRICS.cem_calibration_mse.observe(params.mse);
    params
}

fn gaussian_solve_5x5(a: &[f64], b: &[f64]) -> [f64; 5] {
    let n = 5;
    let mut m = vec![0.0f64; n * (n + 1)];
    for r in 0..n {
        for c in 0..n {
            m[r * (n + 1) + c] = a[r * n + c];
        }
        m[r * (n + 1) + n] = b[r];
    }

    for k in 0..n {
        let mut max_row = k;
        let mut max_val = m[k * (n + 1) + k].abs();
        for i in (k + 1)..n {
            let v = m[i * (n + 1) + k].abs();
            if v > max_val {
                max_val = v;
                max_row = i;
            }
        }
        if max_row != k {
            for j in k..(n + 1) {
                let tmp = m[k * (n + 1) + j];
                m[k * (n + 1) + j] = m[max_row * (n + 1) + j];
                m[max_row * (n + 1) + j] = tmp;
            }
        }

        let pivot = m[k * (n + 1) + k];
        assert!(pivot.abs() > 1e-12, "Singular matrix");
        for j in k..(n + 1) {
            m[k * (n + 1) + j] /= pivot;
        }

        for i in (k + 1)..n {
            let factor = m[i * (n + 1) + k];
            for j in k..(n + 1) {
                m[i * (n + 1) + j] -= factor * m[k * (n + 1) + j];
            }
        }
    }

    let mut x = [0.0f64; 5];
    for i in (0..n).rev() {
        let mut s = m[i * (n + 1) + n];
        for j in (i + 1)..n {
            s -= m[i * (n + 1) + j] * x[j];
        }
        x[i] = s;
    }
    x
}

pub fn run_calibration(samples: &[Sample]) -> CalibrationParams {
    let clean = sanitize(samples);
    let (x_rows, y_vals) = extract_features(&clean);
    fit_params(&x_rows, &y_vals)
}
