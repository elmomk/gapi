/// Color constants matching garmin-grafana dashboard

// Base theme
pub const BG: &str = "#0a0e17";
pub const SURFACE: &str = "#131a2b";
pub const BORDER: &str = "#1e2a42";
pub const TEXT: &str = "#e0e6f0";
pub const DIM: &str = "#6b7a99";
pub const ACCENT: &str = "#00d4aa";

// Status
pub const GOOD: &str = "#00d4aa";
pub const WARN: &str = "#ff6b8a";
pub const INFO: &str = "#4a9eff";

// Heart rate zones
pub const HR_ZONE1: &str = "#4a9eff";
pub const HR_ZONE2: &str = "#00d4aa";
pub const HR_ZONE3: &str = "#ffb347";
pub const HR_ZONE4: &str = "#ff6b8a";
pub const HR_ZONE5: &str = "#ff4444";

// Sleep stages
pub const SLEEP_DEEP: &str = "#042c68";
pub const SLEEP_LIGHT: &str = "#4a9eff";
pub const SLEEP_REM: &str = "#a352cc";
pub const SLEEP_AWAKE: &str = "#ff6b8a";

// Stress
pub const STRESS_LOW: &str = "#1a4a6e";
pub const STRESS_REST: &str = "#1a6e3a";
pub const STRESS_MEDIUM: &str = "#e8a030";
pub const STRESS_HIGH: &str = "#d04050";

// Body battery
pub const BB_CHARGED: &str = "#00d4aa";
pub const BB_DRAINED: &str = "#ff6b8a";

// Activity
pub const ACTIVITY_SEDENTARY: &str = "#4a9eff";
pub const ACTIVITY_ACTIVE: &str = "#ffb347";
pub const ACTIVITY_INTENSE: &str = "#ff6b8a";
pub const ACTIVITY_SLEEP: &str = "#042c68";

// Charts
pub const CHART_GREEN: &str = "#00d4aa";
pub const CHART_RED: &str = "#ff6b8a";
pub const CHART_BLUE: &str = "#4a9eff";
pub const CHART_ORANGE: &str = "#ffb347";
pub const CHART_PURPLE: &str = "#a352cc";
pub const CHART_YELLOW: &str = "#e8d030";

/// Format seconds as "Xh Ym"
pub fn fmt_duration(secs: f64) -> String {
    let h = (secs / 3600.0) as i64;
    let m = ((secs % 3600.0) / 60.0) as i64;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

/// Format seconds as "X.Xh"
pub fn fmt_hours(secs: f64) -> String {
    format!("{:.1}h", secs / 3600.0)
}

/// Format a number with appropriate precision
pub fn fmt_val(v: f64) -> String {
    if v == v.floor() && v.abs() < 100000.0 { format!("{:.0}", v) } else { format!("{:.1}", v) }
}

/// Get HR zone color for a given BPM
pub fn hr_zone_color(bpm: i64) -> &'static str {
    match bpm {
        0..=96 => HR_ZONE1,
        97..=120 => HR_ZONE2,
        121..=140 => HR_ZONE3,
        141..=160 => HR_ZONE4,
        _ => HR_ZONE5,
    }
}

/// Get stress color for a given stress level
pub fn stress_color(level: i64) -> &'static str {
    match level {
        0..=25 => STRESS_REST,
        26..=50 => STRESS_LOW,
        51..=75 => STRESS_MEDIUM,
        _ => STRESS_HIGH,
    }
}

/// Interpolate between two colors based on value in [0, 1]
pub fn lerp_color(low: &str, high: &str, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    let parse = |s: &str, i: usize| u8::from_str_radix(&s[i..i+2], 16).unwrap_or(0);
    let r = (parse(low, 1) as f64 * (1.0 - t) + parse(high, 1) as f64 * t) as u8;
    let g = (parse(low, 3) as f64 * (1.0 - t) + parse(high, 3) as f64 * t) as u8;
    let b = (parse(low, 5) as f64 * (1.0 - t) + parse(high, 5) as f64 * t) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
