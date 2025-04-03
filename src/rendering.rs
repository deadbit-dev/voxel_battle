use raylib::prelude::*;
use crate::state::{GameState, VoxelType};

const VOXEL_SIZE: f32 = 1.0;
const DEBUG_COLOR: Color = Color { r: 0, g: 255, b: 0, a: 128 }; // Semi-transparent green
const GRID_COLOR: Color = Color { r: 0, g: 255, b: 0, a: 10 }; // More transparent green for grid

pub fn render(state: &GameState, rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut d = rl.begin_drawing(thread);
    
    d.clear_background(Color::BLACK);
    
    {
        let mut d3 = d.begin_mode3D(state.camera);
        draw_players(state, &mut d3);
        draw_voxels(state, &mut d3);
        if state.debug_mode {
            draw_debug_bounding_boxes(state, &mut d3);
        }
    }
    
    // Draw controls in bottom left
    let screen_height = d.get_screen_height();
    d.draw_text("Move player by WASD", 10, screen_height - 60, 20, Color::WHITE);
    d.draw_text("Left click to place voxel, Right click to remove", 10, screen_height - 35, 20, Color::WHITE);
    
    if state.debug_mode {
        // Draw debug information in top left
        let mut y_offset = 10;
        for (id, player) in &state.players {
            let debug_text = format!(
                "Player {}: Pos({:.1}, {:.1}, {:.1})",
                id, player.position.x, player.position.y, player.position.z
            );
            d.draw_text(&debug_text, 10, y_offset, 20, Color::GREEN);
            y_offset += 25;
        }
        
        // Draw voxel count
        let voxel_count = state.world.voxels.len();
        d.draw_text(&format!("Voxels: {}", voxel_count), 10, y_offset, 20, Color::GREEN);
    }
}

fn draw_players(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    for player in state.players.values() {
        // Draw solid cube
        d.draw_cube(
            player.position,
            player.size.x,
            player.size.y,
            player.size.z,
            player.color,
        );

        // Draw wireframe
        d.draw_cube_wires(
            player.position,
            player.size.x,
            player.size.y,
            player.size.z,
            Color::BLACK,
        );
    }
}

fn draw_voxels(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    // Draw all voxels
    for voxel in &state.world.voxels {
        let position = Vector3::new(
            voxel.position.x * VOXEL_SIZE,
            voxel.position.y * VOXEL_SIZE,
            voxel.position.z * VOXEL_SIZE,
        );
        let color = match voxel.voxel_type {
            VoxelType::Empty => Color::BLANK,
            VoxelType::Ground => Color::GRAY,
            VoxelType::Wall => Color::DARKGRAY,
            VoxelType::Player => Color::RED,
        };

        // Draw solid cube for non-empty voxels
        if voxel.voxel_type != VoxelType::Empty {
            d.draw_cube(
                position,
                VOXEL_SIZE,
                VOXEL_SIZE,
                VOXEL_SIZE,
                color,
            );
        }

        // Draw wireframe for non-empty voxels
        if voxel.voxel_type != VoxelType::Empty {
            d.draw_cube_wires(
                position,
                VOXEL_SIZE,
                VOXEL_SIZE,
                VOXEL_SIZE,
                Color::BLACK,
            );
        }
    }

    // Draw complete world grid in debug mode
    if state.debug_mode {
        draw_world_grid(state, d);
    }
}

fn draw_world_grid(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    // Create a set of occupied positions for faster lookup
    let occupied_positions: std::collections::HashSet<(i32, i32, i32)> = state.world.voxels
        .iter()
        .map(|v| (v.position.x as i32, v.position.y as i32, v.position.z as i32))
        .collect();

    // Draw wireframes only for empty spaces
    for x in 0..state.world.width {
        for y in 0..state.world.height {
            for z in 0..state.world.depth {
                // Skip if this position is occupied by a voxel
                if occupied_positions.contains(&(x, y, z)) {
                    continue;
                }

                let position = Vector3::new(
                    x as f32 * VOXEL_SIZE,
                    y as f32 * VOXEL_SIZE,
                    z as f32 * VOXEL_SIZE,
                );
                
                d.draw_cube_wires(
                    position,
                    VOXEL_SIZE,
                    VOXEL_SIZE,
                    VOXEL_SIZE,
                    GRID_COLOR,
                );
            }
        }
    }
}

fn draw_debug_bounding_boxes(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    for player in state.players.values() {
        // Calculate voxel-aligned bounding box
        let min_x = (player.position.x / VOXEL_SIZE).floor() as i32;
        let max_x = (player.position.x / VOXEL_SIZE).ceil() as i32;
        let min_y = (player.position.y / VOXEL_SIZE).floor() as i32;
        let max_y = (player.position.y / VOXEL_SIZE).ceil() as i32;
        let min_z = (player.position.z / VOXEL_SIZE).floor() as i32;
        let max_z = (player.position.z / VOXEL_SIZE).ceil() as i32;

        // Calculate center and size of the voxel-aligned bounding box
        let center_x = (min_x + max_x) as f32 * VOXEL_SIZE / 2.0;
        let center_y = (min_y + max_y) as f32 * VOXEL_SIZE / 2.0;
        let center_z = (min_z + max_z) as f32 * VOXEL_SIZE / 2.0;
        let size_x = (max_x - min_x + 1) as f32 * VOXEL_SIZE;
        let size_y = (max_y - min_y + 1) as f32 * VOXEL_SIZE;
        let size_z = (max_z - min_z + 1) as f32 * VOXEL_SIZE;

        // Draw semi-transparent cube
        d.draw_cube(
            Vector3::new(center_x, center_y, center_z),
            size_x,
            size_y,
            size_z,
            DEBUG_COLOR,
        );
        
        // Draw wireframe
        d.draw_cube_wires(
            Vector3::new(center_x, center_y, center_z),
            size_x,
            size_y,
            size_z,
            Color::GREEN,
        );
    }
} 