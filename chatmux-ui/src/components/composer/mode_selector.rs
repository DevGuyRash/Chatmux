//! Composer mode selector (§3.5).
//!
//! Three options: Send (default), Draft Only, Copy Only.

use leptos::prelude::*;

use crate::components::primitives::segmented_control::{Segment, SegmentedControl};

/// Composer send mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerMode {
    Send,
    DraftOnly,
    CopyOnly,
}

impl ComposerMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Send => "Send",
            Self::DraftOnly => "Draft",
            Self::CopyOnly => "Copy",
        }
    }

    pub fn value(&self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::DraftOnly => "draft",
            Self::CopyOnly => "copy",
        }
    }

    pub fn from_value(v: &str) -> Self {
        match v {
            "draft" => Self::DraftOnly,
            "copy" => Self::CopyOnly,
            _ => Self::Send,
        }
    }
}

/// Composer mode selector.
#[component]
pub fn ModeSelector(
    /// Current mode.
    mode: ReadSignal<ComposerMode>,
    /// On change callback.
    on_change: impl Fn(ComposerMode) + 'static + Send + Sync,
) -> impl IntoView {
    let (mode_str, set_mode_str) = signal(mode.get_untracked().value().to_string());

    // Keep mode_str in sync with mode
    Effect::new(move |_| {
        set_mode_str.set(mode.get().value().to_string());
    });

    view! {
        <SegmentedControl
            segments=vec![
                Segment { value: "send".into(), label: "Send".into() },
                Segment { value: "draft".into(), label: "Draft".into() },
                Segment { value: "copy".into(), label: "Copy".into() },
            ]
            selected=mode_str
            on_change=move |v| on_change(ComposerMode::from_value(&v))
        />
    }
}
