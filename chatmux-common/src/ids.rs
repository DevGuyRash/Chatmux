//! Strongly typed identifiers used across the Chatmux backend.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

define_id!(WorkspaceId);
define_id!(BindingId);
define_id!(MessageId);
define_id!(RunId);
define_id!(RoundId);
define_id!(DispatchId);
define_id!(EdgePolicyId);
define_id!(DeliveryCursorId);
define_id!(TemplateId);
define_id!(ExportProfileId);
define_id!(DiagnosticEventId);
