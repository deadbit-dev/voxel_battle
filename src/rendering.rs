use raylib::prelude::*;
use crate::state::{GameState, VoxelType, ShaderType};
use crate::config::{VOXEL_SIZE, DEBUG_COLOR, GRID_COLOR, LIGHT_COLOR};

pub fn render(state: &GameState, rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut d = rl.begin_drawing(thread);
    
    d.clear_background(Color::BLACK);
    
    // Apply outline shader if available
    if let Some(outline_shader) = state.shaders.get(&ShaderType::Outline) {
        unsafe {
            // Set outline shader uniforms
            let texture_size = std::ffi::CString::new("textureSize").unwrap();
            let texture_size_loc = ffi::GetShaderLocation(*outline_shader, texture_size.as_ptr());
            let size = [d.get_screen_width() as f32, d.get_screen_height() as f32];
            ffi::SetShaderValue(*outline_shader, texture_size_loc, size.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC2 as i32);
            
            let outline_size = std::ffi::CString::new("outlineSize").unwrap();
            let outline_size_loc = ffi::GetShaderLocation(*outline_shader, outline_size.as_ptr());
            let size = [2.0f32];
            ffi::SetShaderValue(*outline_shader, outline_size_loc, size.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32);
            
            let outline_color = std::ffi::CString::new("outlineColor").unwrap();
            let outline_color_loc = ffi::GetShaderLocation(*outline_shader, outline_color.as_ptr());
            let color = [1.0f32, 0.0f32, 0.0f32, 1.0f32]; // Red outline
            ffi::SetShaderValue(*outline_shader, outline_color_loc, color.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32);
            
            ffi::BeginShaderMode(*outline_shader);
        }
    }
    
    {
        let mut d3 = d.begin_mode3D(state.camera_state.camera);
        
        // Update shader uniforms
        if let Some(shader) = state.shaders.get(&ShaderType::Lighting) {
            unsafe {
                // Update light position
                let light_position = std::ffi::CString::new("lights[0].position").unwrap();
                let pos_loc = ffi::GetShaderLocation(*shader, light_position.as_ptr());
                let pos = [state.light_source.position.x, state.light_source.position.y, state.light_source.position.z];
                ffi::SetShaderValue(*shader, pos_loc, pos.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3 as i32);
                
                // Update light target
                let light_target = std::ffi::CString::new("lights[0].target").unwrap();
                let target_loc = ffi::GetShaderLocation(*shader, light_target.as_ptr());
                let target = [state.light_source.target.x, state.light_source.target.y, state.light_source.target.z];
                ffi::SetShaderValue(*shader, target_loc, target.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3 as i32);
                
                // Update light color
                let light_color = std::ffi::CString::new("lights[0].color").unwrap();
                let color_loc = ffi::GetShaderLocation(*shader, light_color.as_ptr());
                let color = [
                    state.light_source.color.r as f32 / 255.0,
                    state.light_source.color.g as f32 / 255.0,
                    state.light_source.color.b as f32 / 255.0,
                    state.light_source.color.a as f32 / 255.0
                ];
                ffi::SetShaderValue(*shader, color_loc, color.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32);
                
                // Update light enabled state
                let light_enabled = std::ffi::CString::new("lights[0].enabled").unwrap();
                let enabled_loc = ffi::GetShaderLocation(*shader, light_enabled.as_ptr());
                let enabled = [if state.light_source.enabled { 1i32 } else { 0i32 }];
                ffi::SetShaderValue(*shader, enabled_loc, enabled.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32);
            }
        }
        
        unsafe {
            ffi::BeginShaderMode(state.shaders[&ShaderType::Lighting]);
        }
        draw_players(state, &mut d3);
        draw_voxels(state, &mut d3);
        unsafe {
            ffi::EndShaderMode();
        }
        
        // Draw debug elements after shader mode to make them independent of lighting
        if state.editor.active {
            draw_debug_bounding_boxes(state, &mut d3);
            draw_light_source(state, &mut d3);
            draw_world_grid(state, &mut d3);
            draw_hovered_voxel(state, &mut d3);
        }
    }
    
    // End outline shader if it was applied
    if state.shaders.get(&ShaderType::Outline).is_some() {
        unsafe {
            ffi::EndShaderMode();
        }
    }
    
    let screen_width = d.get_screen_width();
    let screen_height = d.get_screen_height();
    
    // Draw welcome message if no players are ready
    if state.players.is_empty() {
        let text = "Press SPACE (keyboard) or move stick (gamepad) to join the game";
        let text_width = unsafe { ffi::MeasureText(text.as_ptr() as *const i8, 20) };
        d.draw_text(
            text,
            (screen_width - text_width) / 2,
            screen_height / 2,
            20,
            Color::WHITE
        );
    }
    
    // Draw FPS
    let fps = d.get_fps();
    d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::GREEN);
    
    if state.editor.active {
        // Draw debug information in top left
        let mut y_offset = 40;
        
        for (id, player) in &state.players {
            let debug_text = format!(
                "Player {}: Pos({:.1}, {:.1}, {:.1})",
                id, player.position.x, player.position.y, player.position.z
            );
            d.draw_text(&debug_text, 10, y_offset, 20, Color::GREEN);
            y_offset += 30;
        }
        
        // Draw voxel count
        let voxel_count = state.world.voxels.len();
        d.draw_text(&format!("Voxels: {}", voxel_count), 10, y_offset, 20, Color::GREEN);
        y_offset += 30;
        
        // Draw light source info
        let light_text = format!(
            "Light: Pos({:.1}, {:.1}, {:.1}) {}",
            state.light_source.position.x,
            state.light_source.position.y,
            state.light_source.position.z,
            if state.light_source.enabled { "ON" } else { "OFF" }
        );
        d.draw_text(&light_text, 10, y_offset, 20, Color::YELLOW);

        // Draw all controls in bottom left
        let mut control_y = screen_height - 200;
        
        // Camera controls
        d.draw_text("Camera Controls:", 10, control_y, 20, Color::WHITE);
        control_y += 25;
        d.draw_text("Middle Mouse Button - Rotate Camera", 10, control_y, 20, Color::WHITE);
        control_y += 25;
        d.draw_text("Mouse Wheel - Zoom In/Out", 10, control_y, 20, Color::WHITE);
        control_y += 35;

        // Building controls
        d.draw_text("Building Controls:", 10, control_y, 20, Color::WHITE);
        control_y += 25;
        d.draw_text("Right click to switch build/remove mode", 10, control_y, 20, Color::WHITE);
        control_y += 25;
        d.draw_text("Left click to place/remove voxel", 10, control_y, 20, Color::WHITE);
        control_y += 35;

        // General controls
        d.draw_text("Press M to toggle debug mode", 10, control_y, 20, Color::WHITE);
    } else {
        // Draw controls in bottom left when not in debug mode
        d.draw_text("Move player by WASD or Gamepad", 10, screen_height - 60, 20, Color::WHITE);
        d.draw_text("Dash with Shift (keyboard) or Right Trigger (gamepad)", 10, screen_height - 85, 20, Color::WHITE);
        d.draw_text("Press M to toggle debug mode", 10, screen_height - 35, 20, Color::WHITE);
    }
}

fn draw_players(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    for player in state.players.values() {
        // Draw solid cube with current color
        d.draw_cube(
            player.position,
            player.size.x,
            player.size.y,
            player.size.z,
            player.color,
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
            VoxelType::Wall => Color::DARKGRAY
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
    }
}

fn draw_world_grid(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    // Draw world boundaries
    let world_center = Vector3::new(
        (state.world.width as f32 * VOXEL_SIZE) / 2.0,
        (state.world.height as f32 * VOXEL_SIZE) / 2.0,
        (state.world.depth as f32 * VOXEL_SIZE) / 2.0,
    );
    
    d.draw_cube_wires(
        world_center,
        state.world.width as f32 * VOXEL_SIZE,
        state.world.height as f32 * VOXEL_SIZE,
        state.world.depth as f32 * VOXEL_SIZE,
        GRID_COLOR,
    );
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

fn draw_light_source(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    if state.light_source.enabled {
        // Draw a small sphere at the light position
        d.draw_sphere(state.light_source.position, 0.5, LIGHT_COLOR);
        
        // Draw a wireframe sphere around it
        d.draw_sphere_wires(state.light_source.position, 0.5, 8, 8, Color::YELLOW);
        
        // Draw a line to show light direction (from position to target)
        unsafe {
            ffi::DrawLine3D(
                ffi::Vector3 { x: state.light_source.position.x, y: state.light_source.position.y, z: state.light_source.position.z },
                ffi::Vector3 { x: state.light_source.target.x, y: state.light_source.target.y, z: state.light_source.target.z },
                ffi::Color { r: LIGHT_COLOR.r, g: LIGHT_COLOR.g, b: LIGHT_COLOR.b, a: LIGHT_COLOR.a }
            );
        }
    }
}

fn draw_hovered_voxel(state: &GameState, d: &mut RaylibMode3D<RaylibDrawHandle>) {
    if let Some((x, y, z)) = state.editor.hovered_voxel {
        let voxel_size = state.world.voxel_size;
        let position = Vector3::new(
            x as f32 * voxel_size,
            y as f32 * voxel_size,
            z as f32 * voxel_size
        );

        let color = if state.editor.build_mode {
            Color::GREEN
        } else {
            Color::RED
        };

        // Draw a semi-transparent cube at the hovered position
        d.draw_cube_wires(
            position,
            voxel_size,
            voxel_size,
            voxel_size,
            color
        );
    }
} 