//! Extension commands API bridge (keyboard shortcuts).

use crate::bridge::webextension;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct RawKeyboardCommand {
    name: String,
    description: String,
    shortcut: Option<String>,
}

/// A registered keyboard command.
pub struct KeyboardCommand {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
}

pub async fn get_commands() -> Vec<KeyboardCommand> {
    let Ok(value) = webextension::commands_get_all().await else {
        return Vec::new();
    };

    serde_wasm_bindgen::from_value::<Vec<RawKeyboardCommand>>(value)
        .unwrap_or_default()
        .into_iter()
        .map(|command| KeyboardCommand {
            name: command.name,
            description: command.description,
            shortcut: command.shortcut,
        })
        .collect()
}
