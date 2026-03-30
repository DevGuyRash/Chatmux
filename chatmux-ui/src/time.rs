//! Browser-local timestamp formatting helpers for the UI.

use chrono::{DateTime, Datelike, Timelike, Utc};

pub fn format_local_date(dt: DateTime<Utc>) -> String {
    let parts = local_parts(dt);
    format!("{:02}-{:02}", parts.month, parts.day)
}

pub fn format_local_time(dt: DateTime<Utc>) -> String {
    let parts = local_parts(dt);
    format!("{:02}:{:02}", parts.hour, parts.minute)
}

pub fn format_local_time_seconds(dt: DateTime<Utc>) -> String {
    let parts = local_parts(dt);
    format!("{:02}:{:02}:{:02}", parts.hour, parts.minute, parts.second)
}

pub fn format_local_datetime(dt: DateTime<Utc>) -> String {
    let parts = local_parts(dt);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        parts.year,
        parts.month,
        parts.day,
        parts.hour,
        parts.minute,
        parts.second,
        parts.offset_label,
    )
}

pub fn format_local_title_timestamp(dt: DateTime<Utc>) -> String {
    let parts = local_parts(dt);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        parts.year, parts.month, parts.day, parts.hour, parts.minute, parts.second
    )
}

struct LocalParts {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    offset_label: String,
}

#[cfg(target_arch = "wasm32")]
fn local_parts(dt: DateTime<Utc>) -> LocalParts {
    use wasm_bindgen::JsValue;

    let date = js_sys::Date::new(&JsValue::from_f64(dt.timestamp_millis() as f64));
    let offset_minutes = date.get_timezone_offset() as i32;
    let offset_sign = if offset_minutes <= 0 { '+' } else { '-' };
    let offset_total = offset_minutes.abs();
    let offset_hours = offset_total / 60;
    let offset_remainder = offset_total % 60;

    LocalParts {
        year: date.get_full_year() as i32,
        month: (date.get_month() + 1.0) as u32,
        day: date.get_date() as u32,
        hour: date.get_hours() as u32,
        minute: date.get_minutes() as u32,
        second: date.get_seconds() as u32,
        offset_label: format!("UTC{offset_sign}{offset_hours:02}:{offset_remainder:02}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn local_parts(dt: DateTime<Utc>) -> LocalParts {
    LocalParts {
        year: dt.year(),
        month: dt.month(),
        day: dt.day(),
        hour: dt.hour(),
        minute: dt.minute(),
        second: dt.second(),
        offset_label: "UTC".to_owned(),
    }
}

