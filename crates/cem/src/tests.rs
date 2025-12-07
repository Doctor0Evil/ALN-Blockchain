#[cfg(test)]
mod tests {
    use super::cem_calibration_workflow::*;
    use rand::Rng;

    #[test]
    fn test_fit_params_synthetic() {
        let mut rng = rand::thread_rng();
        let alpha = 0.5;
        let beta = 0.1;
        let gamma = 200.0;
        let delta = 1.2;
        let intercept = 50.0;

        let mut samples = Vec::new();
        for _ in 0..100 {
            let gx = rng.gen_range(0.0..10.0);
            let gy = rng.gen_range(0.0..10.0);
            let gz = rng.gen_range(0.0..10.0);
            let ax = rng.gen_range(0.0..100.0);
            let ay = rng.gen_range(0.0..100.0);
            let az = rng.gen_range(0.0..100.0);
            let neural = rng.gen_range(0.0..1.0);
            let f_n = rng.gen_range(0.0..100.0);
            let f_t = rng.gen_range(-10.0..10.0);

            let v2 = (gx * gx + gy * gy + gz * gz) as f64;
            let a2 = (ax * ax + ay * ay + az * az) as f64;
            let force = ((f_n * f_n + f_t * f_t)).sqrt() as f64;
            let neural_s = neural as f64;

            let p = alpha * v2 + beta * a2 + gamma * neural_s + delta * force + intercept;
            let noise: f64 = rng.gen_range(-5.0..5.0);

            samples.push(Sample {
                timestamp_ns: 0,
                subject_id: 1,
                session_id: 1,
                segment_id: "seg".to_string(),
                ax, ay, az, gx, gy, gz, mx: 0.0, my: 0.0, mz: 0.0,
                f_normal: f_n, f_tangential: f_t,
                event_count: 0.0, event_polarity_mean: 0.0,
                eeg_band_power: vec![], emg_rms: 0.0,
                p_mw_measured: (p + noise) as f32,
            });
        }

        let params = run_calibration(&samples);
        // Allow loose tolerance due to noise and finite sample size
        assert!((params.alpha_v2 - alpha).abs() < 0.1);
        assert!((params.beta_a2 - beta).abs() < 0.1);
        assert!((params.gamma_neural - gamma).abs() < 10.0);
        assert!((params.delta_force - delta).abs() < 0.5);
        assert!((params.intercept - intercept).abs() < 5.0);
        assert!(params.mse >= 0.0);
    }

    #[test]
    fn test_deterministic_hash() {
        let mut samples = Vec::new();
        for i in 0..10 {
            samples.push(Sample {
                timestamp_ns: i,
                subject_id: 1,
                session_id: 1,
                segment_id: "seg".to_string(),
                ax: 0.0, ay: 0.0, az: 0.0, gx: 0.0, gy: 0.0, gz: 0.0, mx: 0.0, my: 0.0, mz: 0.0,
                f_normal: 0.0, f_tangential: 0.0, event_count: 0.0, event_polarity_mean: 0.0,
                eeg_band_power: vec![], emg_rms: 0.0, p_mw_measured: 50.0,
            });
        }
        let p1 = run_calibration(&samples);
        let p2 = run_calibration(&samples);
        assert_eq!(p1.hash_hex, p2.hash_hex);
    }
}
