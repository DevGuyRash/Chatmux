//! Extension commands API bridge (keyboard shortcuts).

/// A registered keyboard command.
pub struct KeyboardCommand {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
}

/// TODO(backend): Fetch all registered keyboard commands and their bindings.
/// The actual key combinations are configured through the browser's
/// commands API and can be customized by the user in browser settings.
pub async fn get_commands() -> Vec<KeyboardCommand> {
    log::warn!("STUB: get_commands");
    vec![]
}
