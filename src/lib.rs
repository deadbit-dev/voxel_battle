pub mod state;
pub mod logic;
pub mod rendering;

pub use state::GameState;
pub use logic::{update_player_position, update_player_inputs};
pub use rendering::render;