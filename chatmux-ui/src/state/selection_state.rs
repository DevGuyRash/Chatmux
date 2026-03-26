//! Export and context selection mode state.

use leptos::prelude::*;
use uuid::Uuid;

/// Selection mode type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    /// Not in selection mode.
    None,
    /// Selecting messages for export.
    Export,
    /// Picking messages for context in the composer.
    ContextPick,
}

/// Selection state.
#[derive(Clone, Copy)]
pub struct SelectionState {
    pub mode: ReadSignal<SelectionMode>,
    pub set_mode: WriteSignal<SelectionMode>,
    pub selected_ids: ReadSignal<Vec<Uuid>>,
    pub set_selected_ids: WriteSignal<Vec<Uuid>>,
}

impl SelectionState {
    pub fn toggle(&self, id: Uuid) {
        self.set_selected_ids.update(|ids| {
            if let Some(pos) = ids.iter().position(|&i| i == id) {
                ids.remove(pos);
            } else {
                ids.push(id);
            }
        });
    }

    pub fn clear(&self) {
        self.set_selected_ids.set(Vec::new());
    }

    pub fn is_active(&self) -> bool {
        self.mode.get_untracked() != SelectionMode::None
    }
}

pub fn provide_selection_state() {
    let (mode, set_mode) = signal(SelectionMode::None);
    let (selected_ids, set_selected_ids) = signal(Vec::<Uuid>::new());

    provide_context(SelectionState {
        mode,
        set_mode,
        selected_ids,
        set_selected_ids,
    });
}
