pub mod config;
pub mod state;
pub mod logic;
pub mod rendering;
pub mod utils;

pub use state::GameState;
pub use logic::{init, update};
pub use rendering::render;
pub use config::{SCREEN_WIDTH, SCREEN_HEIGHT, VOXEL_SIZE, DEBUG_COLOR, GRID_COLOR, LIGHT_COLOR, PLAYER_COLORS};