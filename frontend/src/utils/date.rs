use chrono::{DateTime, Utc};

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

pub fn format_time(dt: &DateTime<Utc>) -> String {
    dt.format("%H:%M:%S").to_string()
}

pub fn format_duration_seconds(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
