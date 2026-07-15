//! Scoring constants and recency utilities — BM25 parameters, FSRS-6
//! forgetting curve, and recency/salience boost functions.

pub(crate) const BM25_K1: f64 = 1.2;
pub(crate) const BM25_B: f64 = 0.75;

// ─── FSRS-6 Default Parameters ──────────────────────────────
// From open-spaced-repetition/awesome-fsrs
const FSRS_W: [f64; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001,
    1.8722, 0.1666, 0.796, 1.4835, 0.0614, 0.2629, 1.6483, 0.6014,
    1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

use crate::config::RecencyModel;

/// Compute a recency boost based on days since last update.
/// Models: "fsrs" (default), "linear", "exponential", "none".
/// `stability_days` is the half-life parameter for all models.
pub fn recency_boost(days_since_update: f64, model: &RecencyModel, stability_days: f64) -> f64 {
    if days_since_update <= 0.0 {
        return 1.0;
    }
    if stability_days <= 0.0 {
        return 1.0;
    }
    match model {
        RecencyModel::Fsrs => {
            // FSRS-6 forgetting curve
            let w20 = FSRS_W[20];
            let factor = 0.9_f64.powf(-1.0 / w20) - 1.0;
            let r = (1.0 + factor * days_since_update / stability_days).powf(-w20);
            r.clamp(0.0, 1.0)
        }
        RecencyModel::Linear => (1.0 - days_since_update / stability_days).max(0.0),
        RecencyModel::Exponential => (-days_since_update / stability_days).exp(),
        RecencyModel::None => 1.0,
    }
}

/// Cap total boost from multiple sources (recency × salience) to prevent domination.
pub fn cap_total_boost(recency: f64, salience: f64, max_boost: f64) -> f64 {
    (recency * salience).min(max_boost)
}
