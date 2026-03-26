//! Bridge layer — WebExtension API wrappers.
//!
//! Every function in this module is a TODO stub. The bridge defines
//! the communication boundary between the UI and the background
//! coordinator. The backend engineer will implement the actual
//! message handling on the other side.
//!
//! All functions describe WHAT data flows in and out, without
//! defining specific types, struct names, or message formats.

pub mod clipboard;
pub mod commands;
pub mod messaging;
pub mod permissions;
pub mod storage;
pub mod tabs;
