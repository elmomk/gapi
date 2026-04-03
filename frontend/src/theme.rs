/// Cyberpunk color constants matching life_manager / gorilla_coach

// Base
pub const BG: &str = "#08080f";
pub const SURFACE: &str = "#141428";
pub const BORDER: &str = "#1e1e3a";
pub const TEXT: &str = "#e0e0e8";
pub const DIM: &str = "#5a5a7a";

// Neon accents
pub const ACCENT: &str = "#00f0ff";
pub const GREEN: &str = "#00ff41";
pub const GOOD: &str = "#00ff41";
pub const WARN: &str = "#ff2d78";
pub const INFO: &str = "#00f0ff";

// Metric colors
pub const CHART_GREEN: &str = "#00ff41";
pub const CHART_RED: &str = "#ff2d78";
pub const CHART_BLUE: &str = "#00f0ff";
pub const CHART_ORANGE: &str = "#ff8c00";
pub const CHART_PURPLE: &str = "#bf5af2";
pub const CHART_YELLOW: &str = "#ffdd00";

// Heart rate zones
pub const HR_ZONE1: &str = "#00f0ff";
pub const HR_ZONE2: &str = "#00ff41";
pub const HR_ZONE3: &str = "#ffdd00";
pub const HR_ZONE4: &str = "#ff8c00";
pub const HR_ZONE5: &str = "#ff2d78";

// Sleep stages
pub const SLEEP_DEEP: &str = "#1a1a6e";
pub const SLEEP_LIGHT: &str = "#00f0ff";
pub const SLEEP_REM: &str = "#bf5af2";
pub const SLEEP_AWAKE: &str = "#ff2d78";

// Stress
pub const STRESS_LOW: &str = "#00ff41";
pub const STRESS_REST: &str = "#00f0ff";
pub const STRESS_MEDIUM: &str = "#ff8c00";
pub const STRESS_HIGH: &str = "#ff2d78";

// Body battery
pub const BB_CHARGED: &str = "#00ff41";
pub const BB_DRAINED: &str = "#ff2d78";

// Activity
pub const ACTIVITY_SEDENTARY: &str = "#5a5a7a";
pub const ACTIVITY_ACTIVE: &str = "#ff8c00";
pub const ACTIVITY_INTENSE: &str = "#ff2d78";
pub const ACTIVITY_SLEEP: &str = "#1a1a6e";

pub fn fmt_duration(secs: f64) -> String {
    let h = (secs / 3600.0) as i64;
    let m = ((secs % 3600.0) / 60.0) as i64;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

pub fn fmt_hours(secs: f64) -> String {
    format!("{:.1}h", secs / 3600.0)
}

pub fn fmt_val(v: f64) -> String {
    if v == v.floor() && v.abs() < 100000.0 { format!("{:.0}", v) } else { format!("{:.1}", v) }
}

pub fn hr_zone_color(bpm: i64) -> &'static str {
    match bpm {
        0..=96 => HR_ZONE1,
        97..=120 => HR_ZONE2,
        121..=140 => HR_ZONE3,
        141..=160 => HR_ZONE4,
        _ => HR_ZONE5,
    }
}

pub fn stress_color(level: i64) -> &'static str {
    match level {
        0..=25 => STRESS_REST,
        26..=50 => STRESS_LOW,
        51..=75 => STRESS_MEDIUM,
        _ => STRESS_HIGH,
    }
}

pub fn lerp_color(low: &str, high: &str, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    let parse = |s: &str, i: usize| u8::from_str_radix(&s[i..i+2], 16).unwrap_or(0);
    let r = (parse(low, 1) as f64 * (1.0 - t) + parse(high, 1) as f64 * t) as u8;
    let g = (parse(low, 3) as f64 * (1.0 - t) + parse(high, 3) as f64 * t) as u8;
    let b = (parse(low, 5) as f64 * (1.0 - t) + parse(high, 5) as f64 * t) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
