//! Icon component.
//!
//! Wraps SVG icons from the icon set. Uses Lucide-compatible 24×24 stroke-based
//! paths where available, falling back to placeholder text for any unassigned
//! variants.
//! All icons follow: outlined style, 1.5px stroke weight.

use leptos::prelude::*;

/// Icon identifiers matching the iconography requirements (§6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconKind {
    // Navigation
    Grid,
    ChatBubble,
    GitBranch,
    Document,
    Shield,
    ShieldExclamation,
    Gear,
    ArrowLeft,
    ChevronDown,
    ChevronRight,
    Close,

    // Actions
    Plus,
    Pencil,
    Duplicate,
    ArchiveBox,
    Trash,
    Download,
    Clipboard,
    ClipboardCheck,
    Eye,
    EyeSlash,
    Search,
    Funnel,
    ArrowUp,
    ArrowDown,
    Bookmark,

    // Run Controls
    Play,
    Pause,
    Step,
    Stop,
    StopOctagon,
    Rewind,
    Snowflake,

    // Provider Health
    CircleFilled,
    PencilCircle,
    ArrowOutgoing,
    Spinner,
    CheckCircle,
    BrokenLink,
    Lock,
    PersonX,
    WarningTriangle,
    StopCircle,
    ClockSlash,
    XCircle,
    QuestionCircle,
    HandRaised,

    // Messaging
    PaperPlane,
    DocumentOutline,
    ClipboardOutlined,
    Checkmark,
    Tag,
    Crosshair,
    Pin,
    DragHandle,
    StrikethroughCircle,

    // Export
    DocumentLines,
    CurlyBraces,
    KeyValue,
    FloppyDisk,
    FolderOpen,

    // System
    InfoCircle,
    ExclamationTriangle,
    ExclamationCircle,
    ShieldCheck,
    PersonSilhouette,
    GearSmall,
    PuzzlePiece,
    StackedRectangles,
    BarChart,
    Keyboard,
    ExternalLink,
}

impl IconKind {
    /// Placeholder text character for the icon (fallback when no SVG paths are defined).
    pub fn placeholder(&self) -> &'static str {
        match self {
            Self::Grid => "⊞",
            Self::ChatBubble => "💬",
            Self::GitBranch => "⑂",
            Self::Document => "📄",
            Self::Shield | Self::ShieldCheck => "🛡",
            Self::ShieldExclamation => "⚠",
            Self::Gear | Self::GearSmall => "⚙",
            Self::ArrowLeft => "←",
            Self::ChevronDown => "▾",
            Self::ChevronRight => "▸",
            Self::Close => "✕",
            Self::Plus => "+",
            Self::Pencil | Self::PencilCircle => "✎",
            Self::Duplicate => "⧉",
            Self::ArchiveBox => "📥",
            Self::Trash => "🗑",
            Self::Download => "↓",
            Self::Clipboard | Self::ClipboardOutlined => "📋",
            Self::ClipboardCheck => "✅",
            Self::Eye => "👁",
            Self::EyeSlash => "⊘",
            Self::Search => "🔍",
            Self::Funnel => "▽",
            Self::ArrowUp => "↑",
            Self::ArrowDown => "↓",
            Self::Bookmark => "🔖",
            Self::Play => "▶",
            Self::Pause => "⏸",
            Self::Step => "⏭",
            Self::Stop => "⏹",
            Self::StopOctagon => "⛔",
            Self::Rewind => "⏪",
            Self::Snowflake => "❄",
            Self::CircleFilled => "●",
            Self::ArrowOutgoing => "↗",
            Self::Spinner => "⟳",
            Self::CheckCircle => "✔",
            Self::BrokenLink => "⛓",
            Self::Lock => "🔒",
            Self::PersonX => "👤",
            Self::WarningTriangle | Self::ExclamationTriangle => "⚠",
            Self::StopCircle => "⊘",
            Self::ClockSlash => "⏱",
            Self::XCircle | Self::ExclamationCircle => "✖",
            Self::QuestionCircle => "?",
            Self::HandRaised => "✋",
            Self::PaperPlane => "➤",
            Self::DocumentOutline | Self::DocumentLines => "☰",
            Self::Checkmark => "✓",
            Self::Tag => "🏷",
            Self::Crosshair => "⊕",
            Self::Pin => "📌",
            Self::DragHandle => "⠿",
            Self::StrikethroughCircle => "⊘",
            Self::CurlyBraces => "{ }",
            Self::KeyValue => "≡",
            Self::FloppyDisk => "💾",
            Self::FolderOpen => "📂",
            Self::InfoCircle => "ℹ",
            Self::PersonSilhouette => "👤",
            Self::PuzzlePiece => "🧩",
            Self::StackedRectangles => "▤",
            Self::BarChart => "📊",
            Self::Keyboard => "⌨",
            Self::ExternalLink => "↗",
        }
    }

    /// SVG path `d` attribute strings for this icon (Lucide-compatible, 24×24 viewBox).
    ///
    /// Returns `None` for icons that have no SVG data yet — the `Icon` component
    /// will fall back to `placeholder()` in that case.
    pub fn svg_paths(&self) -> Option<&'static [&'static str]> {
        match self {
            // Navigation
            Self::Grid => Some(&[
                "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z",
            ]),
            Self::ChatBubble => Some(&[
                "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
            ]),
            Self::GitBranch => Some(&[
                "M6 3v12",
                "M18 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
                "M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
                "M18 9a9 9 0 0 1-9 9",
            ]),
            Self::Document => Some(&[
                "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
                "M14 2v6h6",
            ]),
            Self::Shield => Some(&[
                "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
            ]),
            Self::ShieldExclamation => Some(&[
                "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
                "M12 8v4",
                "M12 16h.01",
            ]),
            Self::Gear => Some(&[
                "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            ]),
            Self::ArrowLeft => Some(&[
                "M19 12H5",
                "M12 19l-7-7 7-7",
            ]),
            Self::ChevronDown => Some(&["M6 9l6 6 6-6"]),
            Self::ChevronRight => Some(&["M9 18l6-6-6-6"]),
            Self::Close => Some(&[
                "M18 6L6 18",
                "M6 6l12 12",
            ]),

            // Actions
            Self::Plus => Some(&[
                "M12 5v14",
                "M5 12h14",
            ]),
            Self::Pencil => Some(&[
                "M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z",
            ]),
            Self::Duplicate => Some(&[
                "M16 3H4v13",
                "M8 7h12v14H8z",
            ]),
            Self::ArchiveBox => Some(&[
                "M21 8v13H3V8",
                "M1 3h22v5H1z",
                "M10 12h4",
            ]),
            Self::Trash => Some(&[
                "M3 6h18",
                "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6",
                "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2",
            ]),
            Self::Download => Some(&[
                "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4",
                "M7 10l5 5 5-5",
                "M12 15V3",
            ]),
            Self::Clipboard => Some(&[
                "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2",
                "M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z",
            ]),
            Self::ClipboardCheck => Some(&[
                "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2",
                "M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z",
                "M9 14l2 2 4-4",
            ]),
            Self::Eye => Some(&[
                "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8z",
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            ]),
            Self::EyeSlash => Some(&[
                "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94",
                "M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19",
                "M1 1l22 22",
                "M14.12 14.12a3 3 0 1 1-4.24-4.24",
            ]),
            Self::Search => Some(&[
                "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z",
                "M21 21l-4.35-4.35",
            ]),
            Self::Funnel => Some(&["M22 3H2l8 9.46V19l4 2v-8.54z"]),
            Self::ArrowUp => Some(&[
                "M12 19V5",
                "M5 12l7-7 7 7",
            ]),
            Self::ArrowDown => Some(&[
                "M12 5v14",
                "M19 12l-7 7-7-7",
            ]),
            Self::Bookmark => Some(&[
                "M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z",
            ]),

            // Run Controls
            Self::Play => Some(&["M5 3l14 9-14 9z"]),
            Self::Pause => Some(&[
                "M6 4h4v16H6z",
                "M14 4h4v16h-4z",
            ]),
            Self::Step => Some(&[
                "M5 4l10 8-10 8z",
                "M19 5v14",
            ]),
            Self::Stop => Some(&["M6 4h12v16H6z"]),
            Self::StopOctagon => Some(&[
                "M7.86 2h8.28L22 7.86v8.28L16.14 22H7.86L2 16.14V7.86z",
                "M10 15V9",
                "M14 15V9",
            ]),
            Self::Rewind => Some(&[
                "M11 19l-9-7 9-7z",
                "M22 19l-9-7 9-7z",
            ]),
            Self::Snowflake => Some(&[
                "M12 2v20",
                "M17 7l-10 10",
                "M7 7l10 10",
                "M2 12h20",
                "M4.93 4.93l14.14 14.14",
                "M19.07 4.93L4.93 19.07",
            ]),

            // Provider Health
            Self::CircleFilled => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
            ]),
            Self::PencilCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M15 8l-6 6",
                "M9 14l1.5-5.5L16 7z",
            ]),
            Self::ArrowOutgoing => Some(&[
                "M7 17L17 7",
                "M7 7h10v10",
            ]),
            Self::Spinner => Some(&["M21 12a9 9 0 1 1-6.219-8.56"]),
            Self::CheckCircle => Some(&[
                "M22 11.08V12a10 10 0 1 1-5.93-9.14",
                "M22 4L12 14.01l-3-3",
            ]),
            Self::BrokenLink => Some(&[
                "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71",
                "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71",
            ]),
            Self::Lock => Some(&[
                "M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2z",
                "M7 11V7a5 5 0 0 1 10 0v4",
            ]),
            Self::PersonX => Some(&[
                "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2",
                "M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
                "M17 8l5 5",
                "M22 8l-5 5",
            ]),
            Self::WarningTriangle => Some(&[
                "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z",
                "M12 9v4",
                "M12 17h.01",
            ]),
            Self::StopCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M9 9h6v6H9z",
            ]),
            Self::ClockSlash => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M12 6v6l4 2",
            ]),
            Self::XCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M15 9l-6 6",
                "M9 9l6 6",
            ]),
            Self::QuestionCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3",
                "M12 17h.01",
            ]),
            Self::HandRaised => Some(&[
                "M18 11V6a2 2 0 0 0-4 0",
                "M14 10V4a2 2 0 0 0-4 0v6",
                "M10 10.5V6a2 2 0 0 0-4 0v8",
                "M18 8a2 2 0 0 1 4 0v6a8 8 0 0 1-8 8h-2c-2.83 0-3.36-.94-5.12-2.5L4.26 17a2 2 0 0 1 2.83-2.83L10 16.5",
            ]),

            // Messaging
            Self::PaperPlane => Some(&[
                "M22 2L11 13",
                "M22 2l-7 20-4-9-9-4z",
            ]),
            Self::DocumentOutline => Some(&[
                "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
                "M14 2v6h6",
            ]),
            Self::ClipboardOutlined => Some(&[
                "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2",
                "M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z",
            ]),
            Self::Checkmark => Some(&["M20 6L9 17l-5-5"]),
            Self::Tag => Some(&[
                "M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z",
                "M7 7h.01",
            ]),
            Self::Crosshair => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M22 12h-4",
                "M6 12H2",
                "M12 6V2",
                "M12 22v-4",
            ]),
            Self::Pin => Some(&[
                "M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z",
                "M12 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            ]),
            Self::DragHandle => Some(&[
                "M9 5h.01",
                "M9 12h.01",
                "M9 19h.01",
                "M15 5h.01",
                "M15 12h.01",
                "M15 19h.01",
            ]),
            Self::StrikethroughCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M4.93 4.93l14.14 14.14",
            ]),

            // Export
            Self::DocumentLines => Some(&[
                "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
                "M14 2v6h6",
                "M16 13H8",
                "M16 17H8",
                "M10 9H8",
            ]),
            Self::CurlyBraces => Some(&[
                "M8 21s-4-3-4-9 4-9 4-9",
                "M16 21s4-3 4-9-4-9-4-9",
            ]),
            Self::KeyValue => Some(&[
                "M8 6h13",
                "M8 12h13",
                "M8 18h13",
                "M3 6h.01",
                "M3 12h.01",
                "M3 18h.01",
            ]),
            Self::FloppyDisk => Some(&[
                "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z",
                "M17 21v-8H7v8",
                "M7 3v5h8",
            ]),
            Self::FolderOpen => Some(&[
                "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z",
                "M2 10h20",
            ]),

            // System
            Self::InfoCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M12 16v-4",
                "M12 8h.01",
            ]),
            Self::ExclamationTriangle => Some(&[
                "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z",
                "M12 9v4",
                "M12 17h.01",
            ]),
            Self::ExclamationCircle => Some(&[
                "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z",
                "M12 8v4",
                "M12 16h.01",
            ]),
            Self::ShieldCheck => Some(&[
                "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
                "M9 12l2 2 4-4",
            ]),
            Self::PersonSilhouette => Some(&[
                "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2",
                "M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
            ]),
            Self::GearSmall => Some(&[
                "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            ]),
            Self::PuzzlePiece => Some(&[
                "M19.439 7.85c-.049.322.059.645.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.877-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.877l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.315 8.685a.98.98 0 0 1 .837-.276c.47.07.802.48.968.925a2.501 2.501 0 1 0 3.214-3.214c-.446-.166-.855-.497-.925-.968a.979.979 0 0 1 .276-.837l1.61-1.61A2.404 2.404 0 0 1 12 2c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.877.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.237 3.237c-.464.18-.894.527-.967 1.02z",
            ]),
            Self::StackedRectangles => Some(&[
                "M12 2L2 7l10 5 10-5-10-5z",
                "M2 17l10 5 10-5",
                "M2 12l10 5 10-5",
            ]),
            Self::BarChart => Some(&[
                "M12 20V10",
                "M18 20V4",
                "M6 20v-4",
            ]),
            Self::Keyboard => Some(&[
                "M2 6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z",
                "M6 10h.01",
                "M10 10h.01",
                "M14 10h.01",
                "M18 10h.01",
                "M8 14h8",
            ]),
            Self::ExternalLink => Some(&[
                "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6",
                "M15 3h6v6",
                "M10 14L21 3",
            ]),
        }
    }

    /// Whether this icon should render as filled rather than stroked.
    pub fn is_filled(&self) -> bool {
        matches!(self, Self::CircleFilled | Self::Play)
    }

    /// Whether this icon uses dot-style paths that need a thick stroke to appear
    /// as filled circles (e.g. `DragHandle`).
    pub fn is_dot_style(&self) -> bool {
        matches!(self, Self::DragHandle)
    }
}

/// Icon component.
#[component]
pub fn Icon(
    /// Which icon to display.
    kind: IconKind,
    /// Size in pixels.
    #[prop(default = 16)]
    size: u32,
    /// Optional CSS color override.
    #[prop(optional, into)]
    color: Option<String>,
    /// Optional aria-label.
    #[prop(optional, into)]
    aria_label: Option<String>,
) -> impl IntoView {
    let color_style = color
        .as_ref()
        .map(|c| format!("color: {c};"))
        .unwrap_or_default();

    let has_label = aria_label.is_some();

    match kind.svg_paths() {
        Some(paths) => {
            let fill = if kind.is_filled() { "currentColor" } else { "none" };
            let stroke = if kind.is_filled() { "none" } else { "currentColor" };
            let stroke_width = if kind.is_dot_style() { "3" } else { "1.5" };

            let path_views: Vec<_> = paths
                .iter()
                .map(|d| {
                    view! { <path d=*d /> }
                })
                .collect();

            view! {
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width=size
                    height=size
                    viewBox="0 0 24 24"
                    fill=fill
                    stroke=stroke
                    stroke-width=stroke_width
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="icon"
                    style=format!("flex-shrink: 0; {color_style}")
                    aria-label=aria_label
                    aria-hidden=if has_label { "false" } else { "true" }
                >
                    {path_views}
                </svg>
            }
            .into_any()
        }
        None => {
            let font_size = (size as f32 * 0.75) as u32;
            view! {
                <span
                    class="icon"
                    style=format!(
                        "display: inline-flex; align-items: center; justify-content: center; \
                         width: {size}px; height: {size}px; \
                         font-size: {font_size}px; line-height: 1; \
                         {color_style} flex-shrink: 0;",
                    )
                    aria-label=aria_label
                    aria-hidden=if has_label { "false" } else { "true" }
                >
                    {kind.placeholder()}
                </span>
            }
            .into_any()
        }
    }
}
