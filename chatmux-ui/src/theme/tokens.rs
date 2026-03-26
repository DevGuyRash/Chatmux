//! Rust-side design token constants.
//!
//! These mirror the CSS custom properties from tokens.css for cases where
//! Rust code needs to reference token values directly (e.g., computed
//! inline styles, canvas drawing, or programmatic animations).
//!
//! Most components should use CSS classes and custom properties instead
//! of these constants. Use these only when CSS cannot express the need.

/// Spacing scale values in pixels (§1.3).
pub mod spacing {
    pub const SPACE_0: f64 = 0.0;
    pub const SPACE_1: f64 = 2.0;
    pub const SPACE_2: f64 = 4.0;
    pub const SPACE_3: f64 = 6.0;
    pub const SPACE_4: f64 = 8.0;
    pub const SPACE_5: f64 = 12.0;
    pub const SPACE_6: f64 = 16.0;
    pub const SPACE_7: f64 = 20.0;
    pub const SPACE_8: f64 = 24.0;
    pub const SPACE_9: f64 = 32.0;
    pub const SPACE_10: f64 = 48.0;
}

/// Z-index layer values (§1.7).
pub mod z_index {
    pub const Z_BASE: i32 = 0;
    pub const Z_RAISED: i32 = 10;
    pub const Z_DROPDOWN: i32 = 100;
    pub const Z_OVERLAY: i32 = 200;
    pub const Z_MODAL: i32 = 300;
    pub const Z_TOAST: i32 = 400;
    pub const Z_TOOLTIP: i32 = 500;
}

/// Layout breakpoints.
pub mod breakpoints {
    /// Width threshold below which we use sidebar layout.
    pub const SIDEBAR_MAX_WIDTH: f64 = 500.0;
}

/// Animation durations in milliseconds (§1.6).
pub mod duration {
    pub const INSTANT: u32 = 50;
    pub const FAST: u32 = 100;
    pub const NORMAL: u32 = 200;
    pub const SLOW: u32 = 350;
    pub const GENTLE: u32 = 500;
}
