pub mod filler;
pub mod god_set;
pub mod tanks;
pub mod history;
pub mod arena;
pub mod secure;
pub mod pusoy;

pub use crate::apps::filler::{FillerGlobalState};
pub use crate::apps::god_set::GodSetGlobalState;
pub use crate::apps::tanks::TanksGlobalState;
pub use crate::apps::history::HistoryGlobalState;
pub use crate::apps::arena::ArenaGlobalState;
pub use crate::apps::secure::SecureGlobalState;
pub use crate::apps::pusoy::PusoyGlobalState;