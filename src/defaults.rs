/// Centralized default thresholds and scoring constants.
/// All magic numbers used across the codebase are defined here.

// --- Threshold defaults ---
pub const MAX_LINES: usize = 300;
pub const MAX_DEPTH: usize = 4;
pub const MAX_PARAMS: usize = 4;
pub const MAX_COMPLEXITY: usize = 10;
pub const MAX_COGNITIVE_COMPLEXITY: usize = 15;

// --- Scoring weights (out of 100) ---
pub const WEIGHT_COMPLEXITY: f64 = 35.0;
pub const WEIGHT_ISSUES: f64 = 40.0;
pub const WEIGHT_STRUCTURE: f64 = 25.0;

// --- Complexity scoring ---
/// Cyclomatic complexity threshold before penalty kicks in
pub const CC_THRESHOLD: f64 = 10.0;
/// Penalty multiplier per unit of excess cyclomatic complexity
pub const CC_PENALTY_RATE: f64 = 2.0;
/// Maximum penalty from a single function's cyclomatic complexity
pub const CC_PENALTY_CAP: f64 = 15.0;
/// Cognitive complexity threshold before penalty kicks in
pub const COG_THRESHOLD: f64 = 15.0;
/// Penalty multiplier per unit of excess cognitive complexity
pub const COG_PENALTY_RATE: f64 = 1.5;
/// Maximum penalty from a single function's cognitive complexity
pub const COG_PENALTY_CAP: f64 = 12.0;

// --- Issue scoring ---
/// Penalty per error-severity issue
pub const ERROR_PENALTY: f64 = 8.0;
/// Base penalty for warning-severity issues (scaled by count^WARNING_EXPONENT)
pub const WARNING_PENALTY: f64 = 4.0;
/// Sub-linear exponent for warning count scaling (diminishing returns)
pub const WARNING_EXPONENT: f64 = 0.7;

// --- Structure scoring ---
/// (rule_name, penalty_per_violation, max_penalty)
pub const STRUCTURE_PENALTIES: &[(&str, f64, f64)] = &[
    ("max-lines", 10.0, 12.0),
    ("max-depth", 4.0, 12.0),
    ("max-params", 3.0, 10.0),
];

// --- Project scoring ---
/// Multiplier for issue density penalty (applied to sqrt of total issues)
pub const DENSITY_PENALTY_RATE: f64 = 0.8;
/// Maximum issue density penalty
pub const DENSITY_PENALTY_CAP: f64 = 15.0;

// --- Parse coverage ---
/// Percentage of a file (by bytes) the grammar may fail to parse before the
/// file is treated as *unanalyzed* rather than scored. A file the grammar
/// cannot read produces no functions and no issues, which otherwise yields a
/// near-perfect score — so above this share we refuse to score it at all,
/// exclude it from the project average, and surface it in the summary. A
/// single unsupported construct tends to shred everything after it, so this is
/// deliberately low.
pub const UNANALYZED_MIN_SHARE: f64 = 20.0;

// --- Hotspot scoring ---
/// Cognitive complexity weight relative to cyclomatic in hotspot score
pub const HOTSPOT_COGNITIVE_WEIGHT: f64 = 0.5;
/// Risk threshold for "high" classification
pub const HOTSPOT_RISK_HIGH: f64 = 100.0;
/// Risk threshold for "medium" classification
pub const HOTSPOT_RISK_MEDIUM: f64 = 30.0;

// --- Grade boundaries ---
pub const GRADE_A_MIN: i32 = 90;
pub const GRADE_B_MIN: i32 = 75;
pub const GRADE_C_MIN: i32 = 60;
pub const GRADE_D_MIN: i32 = 40;
