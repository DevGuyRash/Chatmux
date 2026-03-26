//! Icon component.
//!
//! Wraps SVG icons from the icon set. For now uses placeholder text icons
//! that will be replaced with proper SVGs from the icon library.
//! All icons follow: outlined style, 1.5–2px stroke weight.

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
    /// Placeholder text character for the icon.
    /// Will be replaced with actual SVG paths in Phase 9 polish.
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
        .map(|c| format!("color: {c};"))
        .unwrap_or_default();

    let has_label = aria_label.is_some();

    view! {
        <span
            class="icon"
            style=format!(
                "display: inline-flex; align-items: center; justify-content: center; \
                 width: {size}px; height: {size}px; \
                 font-size: {font_size}px; line-height: 1; \
                 {color_style} flex-shrink: 0;",
                font_size = (size as f32 * 0.75) as u32,
            )
            aria-label=aria_label
            aria-hidden=if has_label { "false" } else { "true" }
        >
            {kind.placeholder()}
        </span>
    }
}
