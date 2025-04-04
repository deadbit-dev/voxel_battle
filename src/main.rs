use std::time::Instant;
use raylib::ffi::{SetConfigFlags, ConfigFlags};
use crate::state::GameState;
use crate::logic::{init, update};
use crate::rendering::render;
use crate::config::{SCREEN_WIDTH, SCREEN_HEIGHT};

mod state;
mod logic;
mod rendering;
mod utils;
mod config;

fn main() {
    unsafe {
        SetConfigFlags(ConfigFlags::FLAG_MSAA_4X_HINT as u32);
    }
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Voxel Battle")
        .build();

    let mut state = GameState::default();
    
    init(&mut state);
    
    // rl.set_target_fps(240);
    
    let mut last_update = Instant::now();
    while !rl.window_should_close() {
        let dt = last_update.elapsed().as_secs_f32();
        last_update = Instant::now();

        // Logic 
        update(&mut state, dt);

        // Render 
        render(&state, &mut rl, &thread);
    }
}
