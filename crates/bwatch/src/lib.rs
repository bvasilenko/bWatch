#![forbid(unsafe_code)]

pub mod cli;
pub mod corpus_index;
pub mod error;
pub mod outward_source;
pub mod outward_state;
pub mod poll;
pub mod substrate_input;
pub mod taxonomy;
pub mod update;

pub use cli::{BwatchCli, Cmd, PollArgs};
pub use error::BwatchError;
pub use outward_source::OutwardSourceSubstrate;
pub use outward_state::OutwardSourceState;
pub use taxonomy::FindingCategory;

pub fn routing_key() -> bsuite_core::RoutingKey {
    bsuite_core::RoutingKey::bwatch()
}
