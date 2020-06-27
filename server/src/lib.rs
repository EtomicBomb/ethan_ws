#![feature(try_trait, vec_remove_item, is_sorted)]

mod util;
mod http_handler;
mod server;

pub use server::{Server, PeerId, Disconnect, GlobalState};
