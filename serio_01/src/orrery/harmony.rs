// ═══════════════════════════════════════════════════════════════
// HARMONY -- Pythagorean interval mathematics
//
// Pythagoras discovered that musical consonance arises from simple
// integer ratios. His cosmic extension (Musica Universalis) held
// that the planets produce harmonies based on their orbital ratios.
// Kepler made it rigorous in Harmonices Mundi (1619).
//
// We score orbital configurations by how close each planet pair's
// radius ratio is to a pure Pythagorean interval.
// ═══════════════════════════════════════════════════════════════

/// A Pythagorean harmonic interval with consonance rating.
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub name: &'static str,
    /// Numerator of the ratio (e.g. 3 for 3:2)
    pub num: u32,
    /// Denominator of the ratio (e.g. 2 for 3:2)
    pub den: u32,
    /// Ratio as f64 (always >= 1.0 — we use the larger/smaller)
    pub ratio: f64,
    /// Consonance: 1.0 = perfect, 0.0 = very dissonant
    pub consonance: f64,
}

/// All Pythagorean intervals ordered by consonance (Pythagoras's ranking).
/// Each ratio covers the range up to the next octave (1.0..2.0).
pub const INTERVALS: &[Interval] = &[
    Interval {
        name: "Octave",
        num: 2,
        den: 1,
        ratio: 2.000,
        consonance: 1.00,
    },
    Interval {
        name: "Perfect Fifth",
        num: 3,
        den: 2,
        ratio: 1.500,
        consonance: 0.95,
    },
    Interval {
        name: "Perfect Fourth",
        num: 4,
        den: 3,
        ratio: 1.333,
        consonance: 0.90,
    },
    Interval {
        name: "Major Sixth",
        num: 5,
        den: 3,
        ratio: 1.667,
        consonance: 0.80,
    },
    Interval {
        name: "Major Third",
        num: 5,
        den: 4,
        ratio: 1.250,
        consonance: 0.75,
    },
    Interval {
        name: "Minor Third",
        num: 6,
        den: 5,
        ratio: 1.200,
        consonance: 0.70,
    },
    Interval {
        name: "Minor Sixth",
        num: 8,
        den: 5,
        ratio: 1.600,
        consonance: 0.65,
    },
    Interval {
        name: "Major Second",
        num: 9,
        den: 8,
        ratio: 1.125,
        consonance: 0.40,
    },
    Interval {
        name: "Minor Seventh",
        num: 16,
        den: 9,
        ratio: 1.778,
        consonance: 0.35,
    },
    Interval {
        name: "Major Seventh",
        num: 15,
        den: 8,
        ratio: 1.875,
        consonance: 0.30,
    },
    Interval {
        name: "Tritone",
        num: 45,
        den: 32,
        ratio: 1.406,
        consonance: 0.10,
    },
];

/// Deviation tolerance: within this fraction of the interval ratio is "close"
const TOLERANCE: f64 = 0.04;

/// Compute the harmonic deviation score (penalty) and nearest interval
/// for two orbital radii r_a and r_b.
///
/// Returns (penalty_score: i64, nearest_interval, deviation_fraction)
/// penalty_score is 0 for perfect harmony, up to 1000 for maximum dissonance.
pub fn harmonic_deviation(r_a: f64, r_b: f64) -> (i64, &'static Interval, f64) {
    if r_a <= 0.0 || r_b <= 0.0 {
        return (1000, &INTERVALS[0], 1.0);
    }

    // Normalise: always larger/smaller >= 1.0
    let ratio = if r_a >= r_b { r_a / r_b } else { r_b / r_a };

    // Reduce to [1.0, 2.0) by dividing by octave
    let octaves = ratio.log2().floor();
    let reduced = ratio / 2_f64.powf(octaves);

    // Find nearest interval
    let mut best_interval = &INTERVALS[0];
    let mut best_dev = f64::MAX;

    for interval in INTERVALS {
        let dev = (reduced - interval.ratio).abs() / interval.ratio;
        if dev < best_dev {
            best_dev = dev;
            best_interval = interval;
        }
    }

    // Score: 0 = perfect, 1000 = maximally dissonant
    // Weight by consonance of the nearest interval and deviation magnitude
    let closeness = (1.0 - (best_dev / TOLERANCE).min(1.0)) * best_interval.consonance;
    let penalty = ((1.0 - closeness) * 1000.0) as i64;

    (penalty, best_interval, best_dev)
}

/// Map orbital radius to audio frequency using Kepler's third law.
/// Inner planets (small r) have higher frequency. f ∝ r^(-3/2).
/// Base frequency anchors the outermost orbit to ~55 Hz (low A).
pub fn radius_to_frequency(radius: f64, r_max: f64) -> f64 {
    // Kepler: T² ∝ r³  →  f ∝ 1/T ∝ r^(-3/2)
    let base_hz = 55.0; // outermost planet
    base_hz * (r_max / radius).powf(1.5)
}
