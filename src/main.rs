use std::time::Instant;
use crate::state::GameState;
use crate::logic::{init, update};
use crate::rendering::render;

mod state;
mod logic;
mod rendering;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Voxel Battle")
        .build();

    let mut state = GameState::default();
    
    init(&mut state);
    
    rl.set_target_fps(240);
    
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
