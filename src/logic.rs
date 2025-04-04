use raylib::prelude::*;
use raylib::ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW;
use raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4;
use crate::state::{GameState, PlayerInput, PlayerState, ShaderType, VoxelType, World, Voxel};
use crate::utils::{generate_random_color, lerp_f32};
use crate::config::{GLSL_VERSION, PLAYER_COLORS};


pub fn init(state: &mut GameState) {
    state.world = World::default();
    state.next_player_id = 1; // Start from 1 for gamepad players
    
    // Create floor voxels
    for x in 0..state.world.width {
        for z in 0..state.world.depth {
            set_voxel(&mut state.world, x, 0, z, VoxelType::Ground);
        }
    }
    
    // Initialize keyboard player input but don't create the player yet
    state.player_inputs.insert(0, PlayerInput {
        movement: Vector2::zero(),
        movement_speed: 5.0, // 5 units per second
    });

    // Load basic lighting shader
    let vs_path = std::ffi::CString::new(format!("resources/shaders/glsl{}/lighting.vs", GLSL_VERSION)).unwrap();
    let fs_path = std::ffi::CString::new(format!("resources/shaders/glsl{}/lighting.fs", GLSL_VERSION)).unwrap();
    
    let shader = unsafe {
        ffi::LoadShader(vs_path.as_ptr(), fs_path.as_ptr())
    };
    
    // Get some required shader locations
    let view_pos = std::ffi::CString::new("viewPos").unwrap();
    unsafe {
        let loc = ffi::GetShaderLocation(shader, view_pos.as_ptr());
        *shader.locs.offset(SHADER_LOC_VECTOR_VIEW as isize) = loc;
    }
    
    // Ambient light level (some basic lighting)
    let ambient = std::ffi::CString::new("ambient").unwrap();
    let ambient_loc = unsafe { ffi::GetShaderLocation(shader, ambient.as_ptr()) };
    let ambient_color = [0.3f32, 0.3f32, 0.3f32, 1.0f32]; // Increased ambient light
    unsafe {
        ffi::SetShaderValue(shader, ambient_loc, ambient_color.as_ptr() as *const std::ffi::c_void, SHADER_UNIFORM_VEC4 as i32);
    }

    // Set up light source
    let light_enabled = std::ffi::CString::new("lights[0].enabled").unwrap();
    let light_type = std::ffi::CString::new("lights[0].type").unwrap();
    let light_position = std::ffi::CString::new("lights[0].position").unwrap();
    let light_color = std::ffi::CString::new("lights[0].color").unwrap();
    
    unsafe {
        // Enable light
        let enabled_loc = ffi::GetShaderLocation(shader, light_enabled.as_ptr());
        let enabled = [1i32];
        ffi::SetShaderValue(shader, enabled_loc, enabled.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32);
        
        // Set light type to directional light
        let type_loc = ffi::GetShaderLocation(shader, light_type.as_ptr());
        let light_type = [0i32]; // LIGHT_DIRECTIONAL
        ffi::SetShaderValue(shader, type_loc, light_type.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32);
        
        // Set light target (direction)
        let light_target = std::ffi::CString::new("lights[0].target").unwrap();
        let target_loc = ffi::GetShaderLocation(shader, light_target.as_ptr());
        let target = [state.light_source.target.x, state.light_source.target.y, state.light_source.target.z];
        ffi::SetShaderValue(shader, target_loc, target.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3 as i32);
        
        // Set light position (still needed for directional light)
        let pos_loc = ffi::GetShaderLocation(shader, light_position.as_ptr());
        let pos = [state.light_source.position.x, state.light_source.position.y, state.light_source.position.z];
        ffi::SetShaderValue(shader, pos_loc, pos.as_ptr() as *const std::ffi::c_void, ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3 as i32);
        
        // Set light color with reduced intensity
        let color_loc = ffi::GetShaderLocation(shader, light_color.as_ptr());
        let color = [
            state.light_source.color.r as f32 / 255.0 * 0.8, // Reduced intensity
            state.light_source.color.g as f32 / 255.0 * 0.8,
            state.light_source.color.b as f32 / 255.0 * 0.8,
            state.light_source.color.a as f32 / 255.0
        ];
        ffi::SetShaderValue(shader, color_loc, color.as_ptr() as *const std::ffi::c_void, SHADER_UNIFORM_VEC4 as i32);
    }

    state.shaders.insert(ShaderType::Lighting, shader);
}

pub fn update(state: &mut GameState, delta: f32) {
    // Toggle debug mode with M
    unsafe {
        if ffi::IsKeyPressed(KeyboardKey::KEY_M as i32) {
            state.editor.active = !state.editor.active;
            
            if state.editor.active {
                // Save camera state before entering debug mode
                state.editor.camera.game_camera = Some(state.camera_state.camera);
                state.editor.camera.game_camera_offset = Some(state.camera_state.offset);
                state.editor.camera.game_camera_height = Some(state.camera_state.height);
                state.editor.camera.game_camera_angle = Some(state.camera_state.angle);
            }
        }
    }

    update_player_inputs(state);
    update_player_position(state, delta);
    handle_voxel_input(state);
    update_camera(state, delta);
}

fn update_player_inputs(state: &mut GameState) {
    // Get all currently used colors before any mutable borrows
    let used_colors: Vec<Color> = state.players.values()
        .map(|p| p.color)
        .collect();

    // Update inputs for all players
    for (id, input) in state.player_inputs.iter_mut() {
        if *id == 0 {
            // First player uses keyboard
            let mut movement = Vector2::zero();
            unsafe {
                if ffi::IsKeyDown(KeyboardKey::KEY_D as i32) {
                    movement.x += 1.0;
                }
                if ffi::IsKeyDown(KeyboardKey::KEY_A as i32) {
                    movement.x -= 1.0;
                }
                if ffi::IsKeyDown(KeyboardKey::KEY_S as i32) {
                    movement.y += 1.0;
                }
                if ffi::IsKeyDown(KeyboardKey::KEY_W as i32) {
                    movement.y -= 1.0;
                }
            }
            input.movement = movement;

            // Check for space key to spawn keyboard player
            if !state.players.contains_key(id) && state.players.len() < 5 && unsafe { ffi::IsKeyPressed(KeyboardKey::KEY_SPACE as i32) } {
                if let Some(color) = generate_random_color(&used_colors, &PLAYER_COLORS) {
                    let mut player = PlayerState::default();
                    player.position.x = state.world.width as f32 / 2.0; // Center X
                    player.position.y = 1.0; // Set player 1 unit above the floor
                    player.position.z = state.world.depth as f32 / 2.0; // Center Z
                    player.color = color;
                    player.original_color = color;
                    player.is_ready = true;
                    state.players.insert(*id, player);
                }
            }
        } else {
            // Other players use gamepads
            let gamepad_id = (*id - 1) as i32; // Convert player ID to gamepad ID
            unsafe {
                // Check if gamepad is available and connected
                let is_available = ffi::IsGamepadAvailable(gamepad_id);
                if is_available {
                    // Try different axis indices for Xbox controller
                    let axis_x = ffi::GetGamepadAxisMovement(gamepad_id, 0); // Left stick X
                    let axis_y = ffi::GetGamepadAxisMovement(gamepad_id, 1); // Left stick Y
                    
                    // Simple deadzone check
                    let deadzone = 0.1;
                    let mut movement = Vector2::zero();
                    
                    if axis_x.abs() > deadzone {
                        movement.x = axis_x;
                    }
                    if axis_y.abs() > deadzone {
                        movement.y = axis_y;
                    }
                    
                    // Normalize the vector if it's not zero
                    if movement.x != 0.0 || movement.y != 0.0 {
                        let length = (movement.x * movement.x + movement.y * movement.y).sqrt();
                        movement.x /= length;
                        movement.y /= length;
                    }
                    
                    // Set same movement speed as keyboard players
                    input.movement_speed = 5.0;
                    input.movement = movement;

                    // Check for any gamepad input to spawn gamepad player
                    if !state.players.contains_key(id) && state.players.len() < 5 && (movement.x != 0.0 || movement.y != 0.0) {
                        if let Some(color) = generate_random_color(&used_colors, &PLAYER_COLORS) {
                            let mut player = PlayerState::default();
                            player.position.x = state.world.width as f32 / 2.0; // Center X
                            player.position.y = 1.0; // Set player 1 unit above the floor
                            player.position.z = state.world.depth as f32 / 2.0; // Center Z
                            player.color = color;
                            player.original_color = color;
                            player.is_ready = true;
                            state.players.insert(*id, player);
                        }
                    }
                }
            }
        }
    }

    // Check for new gamepads only if we haven't reached the maximum number of players
    if state.next_player_id < 5 && state.players.len() < 5 { // Allow up to 4 gamepad players (IDs 1-4)
        for i in 0..4 {
            unsafe {
                let is_available = ffi::IsGamepadAvailable(i);
                if is_available {
                    let player_id = i + 1; // Gamepad 0 -> Player 1, Gamepad 1 -> Player 2, etc.
                    if !state.player_inputs.contains_key(&player_id) {
                        println!("New gamepad detected: {} -> Player {}", i, player_id);
                        
                        // Create player input with same speed as keyboard players
                        state.player_inputs.insert(player_id, PlayerInput {
                            movement: Vector2::zero(),
                            movement_speed: 5.0, // Same speed as keyboard players
                        });
                    }
                }
            }
        }
    }
}

fn check_voxel_collision(world: &World, position: Vector3, size: Vector3) -> Option<Voxel> {
    let voxel_size = world.voxel_size;
    
    // Calculate the player's bounding box in voxel coordinates
    let min_x = (position.x / voxel_size).floor() as i32;
    let max_x = (position.x / voxel_size).ceil() as i32;
    let min_y = (position.y / voxel_size).floor() as i32;
    let max_y = (position.y / voxel_size).ceil() as i32;
    let min_z = (position.z / voxel_size).floor() as i32;
    let max_z = (position.z / voxel_size).ceil() as i32;

    // Check for ground beneath the player
    let player_bottom = position.y - size.y/2.0;
    let player_bottom_voxel_y = (player_bottom / voxel_size).floor() as i32;
    
    // Check all voxels under the player's feet
    let mut has_ground = true;
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let voxel_type = get_voxel(world, x, player_bottom_voxel_y, z);
            if voxel_type != VoxelType::Ground {
                has_ground = false;
                break;
            }
        }
        if !has_ground {
            break;
        }
    }
    
    if !has_ground {
        // Return the first non-ground voxel found
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let voxel_type = get_voxel(world, x, player_bottom_voxel_y, z);
                if voxel_type != VoxelType::Ground {
                    return Some(Voxel {
                        position: Vector3::new(x as f32, player_bottom_voxel_y as f32, z as f32),
                        voxel_type
                    });
                }
            }
        }
    }

    // Check all voxels in the player's bounding box for wall collisions
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                let voxel_type = get_voxel(world, x, y, z);
                // Only check for Wall collisions, ignore Ground
                if voxel_type == VoxelType::Wall {
                    // Calculate the actual voxel position
                    let voxel_pos = Vector3::new(
                        x as f32 * voxel_size,
                        y as f32 * voxel_size,
                        z as f32 * voxel_size
                    );
                    
                    // Check if the player is actually inside the voxel
                    let dx = (position.x - voxel_pos.x).abs();
                    let dy = (position.y - voxel_pos.y).abs();
                    let dz = (position.z - voxel_pos.z).abs();
                    
                    // Collision check with player's size for each axis
                    if dx < size.x/2.0 + voxel_size/2.0 &&
                       dy < size.y/2.0 + voxel_size/2.0 &&
                       dz < size.z/2.0 + voxel_size/2.0 {
                        return Some(Voxel {
                            position: Vector3::new(x as f32, y as f32, z as f32),
                            voxel_type
                        });
                    }
                }
            }
        }
    }
    None
}

fn update_player_position(state: &mut GameState, delta: f32) {
    for (id, input) in state.player_inputs.iter() {
        if let Some(player) = state.players.get_mut(id) {
            // Update dash cooldown
            if player.dash_cooldown > 0.0 {
                player.dash_cooldown -= delta;
            }

            // Check for dash input based on player type
            let is_dash_pressed = if *id == 0 {
                // Keyboard player (ID 0) uses Shift key
                unsafe { ffi::IsKeyPressed(KeyboardKey::KEY_LEFT_SHIFT as i32) }
            } else {
                // Gamepad players use right trigger
                let gamepad_id = (*id - 1) as i32;
                unsafe { ffi::IsGamepadButtonPressed(gamepad_id, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1 as i32) }
            };
            
            // Start dash if dash button is pressed, not already dashing, and cooldown is ready
            if is_dash_pressed && !player.is_dashing && player.dash_cooldown <= 0.0 && input.movement.length() > 0.0 {
                player.is_dashing = true;
                player.dash_cooldown = 0.8; // Cooldown time
                // Normalize the movement vector to ensure consistent dash speed in all directions
                let normalized_movement = input.movement.normalized();
                player.dash_direction = Vector3::new(normalized_movement.x, 0.0, normalized_movement.y);
                // Store current velocity before dash
                player.pre_dash_velocity = player.velocity;
                // Store original color
                player.original_color = player.color;
                // Set initial dash velocity
                player.velocity = player.dash_direction * 15.0;
            }

            // Update color during dash
            if player.is_dashing {
                let progress = (0.8 - player.dash_cooldown) / 0.8; // Progress from 0 to 1
                if progress < 0.1 {
                    // Smooth transition to white at the start
                    let t = progress * 10.0; // Scale to 0-1 range
                    player.color = Color {
                        r: ((player.original_color.r as f32 * (1.0 - t) + 255.0 * t).min(255.0)) as u8,
                        g: ((player.original_color.g as f32 * (1.0 - t) + 255.0 * t).min(255.0)) as u8,
                        b: ((player.original_color.b as f32 * (1.0 - t) + 255.0 * t).min(255.0)) as u8,
                        a: player.original_color.a,
                    };
                } else {
                    // Smooth transition back to original color
                    let t = (progress - 0.1) / 0.9; // Scale to 0-1 range
                    player.color = Color {
                        r: ((255.0 * (1.0 - t) + player.original_color.r as f32 * t).min(255.0)) as u8,
                        g: ((255.0 * (1.0 - t) + player.original_color.g as f32 * t).min(255.0)) as u8,
                        b: ((255.0 * (1.0 - t) + player.original_color.b as f32 * t).min(255.0)) as u8,
                        a: player.original_color.a,
                    };
                }
            } else if player.dash_cooldown > 0.0 {
                // Continue smooth transition back to original color during cooldown
                let progress = (0.8 - player.dash_cooldown) / 0.8; // Progress from 0 to 1
                let t = progress; // Scale to 0-1 range
                player.color = Color {
                    r: ((255.0 * (1.0 - t) + player.original_color.r as f32 * t).min(255.0)) as u8,
                    g: ((255.0 * (1.0 - t) + player.original_color.g as f32 * t).min(255.0)) as u8,
                    b: ((255.0 * (1.0 - t) + player.original_color.b as f32 * t).min(255.0)) as u8,
                    a: player.original_color.a,
                };
            } else {
                // Reset to original color when cooldown is complete
                player.color = player.original_color;
            }

            // Calculate target velocity based on input
            let target_velocity = if player.is_dashing {
                // During dash, maintain dash direction and speed
                player.dash_direction * 20.0 // Increased from 20.0 to 30.0 for faster dash
            } else {
                // Normal movement
                Vector3::new(
                    input.movement.x * input.movement_speed * 1.5,
                    0.0,
                    input.movement.y * input.movement_speed * 1.5
                )
            };

            // Calculate acceleration based on whether we're speeding up or slowing down
            let acceleration_rate = if target_velocity.length() > 0.0 {
                3.0 // Increased from 1.5 to 3.0 for faster acceleration
            } else {
                3.0 // Increased from 3.0 to 3.0 for faster deceleration
            };

            // Update velocity with acceleration (only if not dashing)
            if !player.is_dashing {
                let acceleration = (target_velocity - player.velocity) * acceleration_rate;
                player.velocity += acceleration * delta;
            }
            
            // Apply friction when no input is given and not dashing
            if target_velocity.length() == 0.0 && !player.is_dashing {
                let friction = 5.0; // Lower friction for smoother stopping
                player.velocity = player.velocity.lerp(Vector3::zero(), friction * delta);
            }

            // End dash after 0.2 seconds
            if player.is_dashing {
                player.dash_cooldown -= delta;
                if player.dash_cooldown <= 0.6 { // Dash duration is 0.2 seconds
                    player.is_dashing = false;
                    // Calculate interpolation factor based on remaining cooldown
                    // This will give us a smooth transition from dash speed to pre-dash speed
                    let transition_time = 0.03; // Reduced from 0.05 to 0.03 seconds for faster transition
                    let transition_progress = (0.6 - player.dash_cooldown) / transition_time;
                    let t = transition_progress.min(1.0); // Clamp to 1.0
                    
                    // Smoothly interpolate between dash velocity and pre-dash velocity
                    player.velocity = player.velocity.lerp(player.pre_dash_velocity, t * 5.0); // Increased from 3.0 to 5.0 for faster transition
                }
            }

            // Calculate movement based on velocity
            let movement = player.velocity * delta;

            if movement.length() > 0.0 {
                let new_position = player.position + movement;
                let collision = check_voxel_collision(&state.world, new_position, player.size);
                if collision.is_none() {
                    player.position = new_position;
                } else {
                    // Stop movement in the direction of collision
                    player.velocity = Vector3::zero();
                    // Also stop dash if we hit something
                    player.is_dashing = false;
                }
            }
        }
    }
}

fn is_voxel_occupied_by_player(state: &GameState, x: i32, y: i32, z: i32) -> bool {
    let voxel_size = state.world.voxel_size;
    let voxel_pos = Vector3::new(
        x as f32 * voxel_size,
        y as f32 * voxel_size,
        z as f32 * voxel_size
    );
    
    for player in state.players.values() {
        // Check if player's bounding box overlaps with the voxel
        let dx = (player.position.x - voxel_pos.x).abs();
        let dy = (player.position.y - voxel_pos.y).abs();
        let dz = (player.position.z - voxel_pos.z).abs();
        
        if dx < player.size.x/2.0 + voxel_size/2.0 &&
           dy < player.size.y/2.0 + voxel_size/2.0 &&
           dz < player.size.z/2.0 + voxel_size/2.0 {
            return true;
        }
    }
    false
}

fn handle_voxel_input(state: &mut GameState) {
    let mouse_pos = unsafe { ffi::GetMousePosition() };
    let ray: Ray = unsafe { ffi::GetScreenToWorldRay(mouse_pos, state.camera_state.camera.into()).into() };
    
    let voxel_size = state.world.voxel_size;
    let mut closest_collision: Option<(i32, i32, i32, f32, Vector3)> = None;

    // First check collisions with existing voxels
    for voxel in &state.world.voxels {
        if voxel.voxel_type == VoxelType::Empty {
            continue;
        }

        let x = voxel.position.x as i32;
        let y = voxel.position.y as i32;
        let z = voxel.position.z as i32;

        // Create bounding box for the voxel
        let min = Vector3::new(
            x as f32 * voxel_size,
            y as f32 * voxel_size,
            z as f32 * voxel_size
        );
        let max = Vector3::new(
            (x + 1) as f32 * voxel_size,
            (y + 1) as f32 * voxel_size,
            (z + 1) as f32 * voxel_size
        );
        let bbox = BoundingBox::new(min, max);

        // Check ray collision with the voxel's bounding box
        let collision: RayCollision = unsafe { ffi::GetRayCollisionBox(ray.into(), bbox.into()).into() };
        if collision.hit {
            // Update closest collision if this one is closer
            if let Some((_, _, _, current_dist, _)) = closest_collision {
                if collision.distance < current_dist {
                    closest_collision = Some((x, y, z, collision.distance, collision.normal));
                }
            } else {
                closest_collision = Some((x, y, z, collision.distance, collision.normal));
            }
        }
    }

    // If no collision with existing voxels, check for ground layer
    if closest_collision.is_none() {
        // Calculate where the ray would hit the ground plane (y=0)
        let t = -ray.position.y / ray.direction.y;
        if t > 0.0 {
            let hit_point = ray.position + ray.direction * t;
            let x = (hit_point.x / voxel_size).floor() as i32;
            let z = (hit_point.z / voxel_size).floor() as i32;
            
            if x >= 0 && x < state.world.width && z >= 0 && z < state.world.depth {
                closest_collision = Some((x, 0, z, t, Vector3::new(0.0, 1.0, 0.0)));
            }
        }
    }

    // Toggle build mode with right click
    if unsafe { ffi::IsMouseButtonPressed(MouseButton::MOUSE_BUTTON_RIGHT as i32) } {
        state.editor.build_mode = !state.editor.build_mode;
    }

    // Store the hovered position for rendering
    if let Some((x, y, z, _, normal)) = closest_collision {
        if state.editor.build_mode {
            // In build mode, show where the new voxel will be placed
            let (new_x, new_y, new_z) = if get_voxel(&state.world, x, y, z) == VoxelType::Empty {
                (x, y, z)
            } else {
                // If clicking on an existing voxel, show where the new one will be placed
                if normal.x > 0.5 { (x + 1, y, z) }
                else if normal.x < -0.5 { (x - 1, y, z) }
                else if normal.y > 0.5 { (x, y + 1, z) }
                else if normal.y < -0.5 { (x, y - 1, z) }
                else if normal.z > 0.5 { (x, y, z + 1) }
                else { (x, y, z - 1) }
            };
            state.editor.hovered_voxel = Some((new_x, new_y, new_z));
        } else {
            // In remove mode, show the voxel that will be removed
            state.editor.hovered_voxel = Some((x, y, z));
        }
    } else {
        state.editor.hovered_voxel = None;
    }

    // Only handle voxel placement/removal in debug mode
    if !state.editor.active {
        return;
    }

    if let Some((x, y, z, _, normal)) = closest_collision {
        // Check if there's already a voxel at this position
        let existing_voxel = get_voxel(&state.world, x, y, z);
        
        if unsafe { ffi::IsMouseButtonPressed(MouseButton::MOUSE_BUTTON_LEFT as i32) } {
            if state.editor.build_mode {
                // Build mode - place new voxels
                if existing_voxel == VoxelType::Empty {
                    // Check if the new voxel position overlaps with any player
                    if !is_voxel_occupied_by_player(state, x, y, z) {
                        // Get player height (assuming first player)
                        let player_height = if let Some(player) = state.players.get(&0) {
                            player.position.y
                        } else {
                            1.0 // Default height if no player
                        };
                        
                        // Check if this was a floor voxel (y = 0) or if it's below player height
                        let voxel_type = if y == 0 || (y as f32) < player_height {
                            VoxelType::Ground
                        } else {
                            VoxelType::Wall
                        };
                        
                        set_voxel(&mut state.world, x, y, z, voxel_type);
                    }
                } else {
                    // If clicking on an existing voxel, try to place a new one based on the clicked face
                    let (new_x, new_y, new_z) = if normal.x > 0.5 { (x + 1, y, z) }
                        else if normal.x < -0.5 { (x - 1, y, z) }
                        else if normal.y > 0.5 { (x, y + 1, z) }
                        else if normal.y < -0.5 { (x, y - 1, z) }
                        else if normal.z > 0.5 { (x, y, z + 1) }
                        else { (x, y, z - 1) };

                    if new_x >= 0 && new_x < state.world.width &&
                       new_y >= 0 && new_y < state.world.height &&
                       new_z >= 0 && new_z < state.world.depth &&
                       get_voxel(&state.world, new_x, new_y, new_z) == VoxelType::Empty &&
                       !is_voxel_occupied_by_player(state, new_x, new_y, new_z) {
                        // Get player height (assuming first player)
                        let player_height = if let Some(player) = state.players.get(&0) {
                            player.position.y
                        } else {
                            1.0 // Default height if no player
                        };
                        
                        // Set voxel type based on height relative to player
                        let voxel_type = if new_y == 0 || (new_y as f32) < player_height {
                            VoxelType::Ground
                        } else {
                            VoxelType::Wall
                        };
                        
                        set_voxel(&mut state.world, new_x, new_y, new_z, voxel_type);
                    }
                }
            } else {
                // Remove mode - remove existing voxels
                if existing_voxel != VoxelType::Empty && !is_voxel_occupied_by_player(state, x, y, z) {
                    set_voxel(&mut state.world, x, y, z, VoxelType::Empty);
                }
            }
        }
        
        // Check for continuous removal with Ctrl+left click
        let is_ctrl_pressed = unsafe { ffi::IsKeyDown(KeyboardKey::KEY_LEFT_CONTROL as i32) };
        let is_left_pressed = unsafe { ffi::IsMouseButtonDown(MouseButton::MOUSE_BUTTON_LEFT as i32) };
        if !state.editor.build_mode && is_ctrl_pressed && is_left_pressed {
            if existing_voxel != VoxelType::Empty && !is_voxel_occupied_by_player(state, x, y, z) {
                set_voxel(&mut state.world, x, y, z, VoxelType::Empty);
            }
        }
    }
}

fn update_camera(state: &mut GameState, delta: f32) {
    // Handle transition from debug mode
    if let Some(pre_camera) = &state.editor.camera.game_camera {
        let transition_speed = 5.0 * delta;
        
        // Smoothly transition camera position and target
        state.camera_state.camera.position = state.camera_state.camera.position.lerp(
            pre_camera.position,
            transition_speed
        );
        state.camera_state.camera.target = state.camera_state.camera.target.lerp(
            pre_camera.target,
            transition_speed
        );
        
        if let Some(pre_offset) = &state.editor.camera.game_camera_offset {
            state.camera_state.offset = state.camera_state.offset.lerp(
                *pre_offset,
                transition_speed
            );
        }
        
        if let Some(pre_height) = &state.editor.camera.game_camera_height {
            state.camera_state.height = lerp_f32(state.camera_state.height, *pre_height, transition_speed);
        }
        
        if let Some(pre_angle) = &state.editor.camera.game_camera_angle {
            state.camera_state.angle = lerp_f32(state.camera_state.angle, *pre_angle, transition_speed);
        }
        
        // Only clear pre-debug state and allow normal camera logic once transition is complete
        if (state.camera_state.camera.position - pre_camera.position).length() < 0.1 {
            state.editor.camera.game_camera = None;
            state.editor.camera.game_camera_offset = None;
            state.editor.camera.game_camera_height = None;
            state.editor.camera.game_camera_angle = None;
        } else {
            return;
        }
    }

    // Debug mode camera controls
    if state.editor.active {
        // Set center point to the center of the world at y=0
        state.editor.camera.center = Vector3::new(
            state.world.width as f32 * state.world.voxel_size / 2.0,
            0.0,
            state.world.depth as f32 * state.world.voxel_size / 2.0
        );

        // Handle mouse wheel zoom in debug mode
        let wheel_move = unsafe { ffi::GetMouseWheelMove() as f32 };
        if wheel_move != 0.0 {
            state.editor.camera.distance = (state.editor.camera.distance * (1.0 - wheel_move * 0.1)).clamp(5.0, 50.0);
        }

        // Handle middle mouse button rotation
        if unsafe { ffi::IsMouseButtonDown(MouseButton::MOUSE_BUTTON_MIDDLE as i32) } {
            let mouse_delta = unsafe { ffi::GetMouseDelta() };
            state.editor.camera.rotation.x -= mouse_delta.x * 0.01;
            state.editor.camera.rotation.y = (state.editor.camera.rotation.y + mouse_delta.y * 0.01).clamp(-1.5, 1.5);
        }

        // Calculate camera position based on rotation and distance
        let yaw = state.editor.camera.rotation.x;
        let pitch = state.editor.camera.rotation.y;
        
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        // Calculate camera position using spherical coordinates
        let camera_x = state.editor.camera.center.x + state.editor.camera.distance * cos_pitch * sin_yaw;
        let camera_y = state.editor.camera.center.y + state.editor.camera.distance * sin_pitch;
        let camera_z = state.editor.camera.center.z + state.editor.camera.distance * cos_pitch * cos_yaw;

        // Smoothly transition to debug camera position and target
        let transition_speed = 5.0 * delta;
        state.camera_state.camera.position = state.camera_state.camera.position.lerp(
            Vector3::new(camera_x, camera_y, camera_z),
            transition_speed
        );
        state.camera_state.camera.target = state.camera_state.camera.target.lerp(
            state.editor.camera.center,
            transition_speed
        );
    } else {
        // Game mode camera controls
        if !state.players.is_empty() {
            // Calculate center point between all players
            let mut center = Vector3::zero();
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            
            for player in state.players.values() {
                center += player.position;
                min_x = min_x.min(player.position.x);
                max_x = max_x.max(player.position.x);
                min_z = min_z.min(player.position.z);
                max_z = max_z.max(player.position.z);
            }
            
            center /= state.players.len() as f32;
            
            // Calculate required distance to see all players
            let width = max_x - min_x;
            let depth = max_z - min_z;
            let max_dimension = width.max(depth);
            
            // Calculate target height and distance based on player spread
            let target_height = (max_dimension * 0.5).max(15.0).min(30.0);
            let target_distance = (max_dimension * 0.7).max(20.0).min(40.0);
            
            // Smoothly adjust camera height and distance with slower speed
            let height_transition_speed = 1.0 * delta;
            let distance_transition_speed = 1.0 * delta;
            state.camera_state.height = lerp_f32(state.camera_state.height, target_height, height_transition_speed);
            state.camera_state.offset.z = lerp_f32(state.camera_state.offset.z, target_distance, distance_transition_speed);
            
            // Calculate target camera position
            let target_camera_pos = center + state.camera_state.offset;
            
            // Smoothly interpolate camera position and target with delay
            let position_transition_speed = 2.0 * delta;
            let target_transition_speed = 2.0 * delta;
            
            state.camera_state.camera.position = state.camera_state.camera.position.lerp(
                Vector3::new(target_camera_pos.x, state.camera_state.height, target_camera_pos.z),
                position_transition_speed
            );
            
            // Always look at the center of the player group
            state.camera_state.camera.target = state.camera_state.camera.target.lerp(
                center,
                target_transition_speed
            );
        }
    }
}

fn is_valid_position(world: &World, x: i32, y: i32, z: i32) -> bool {
    x >= 0 && x < world.width && y >= 0 && y < world.height && z >= 0 && z < world.depth
}

fn get_voxel(world: &World, x: i32, y: i32, z: i32) -> VoxelType {
    if is_valid_position(world, x, y, z) {
        if let Some(voxel) = world.voxels.iter().find(|v| v.position.x == x as f32 && v.position.y == y as f32 && v.position.z == z as f32) {
            voxel.voxel_type
        } else {
            VoxelType::Empty
        }
    } else {
        VoxelType::Empty
    }
}

fn set_voxel(world: &mut World, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
    if is_valid_position(world, x, y, z) {
        if let Some(voxel) = world.voxels.iter_mut().find(|v| v.position.x == x as f32 && v.position.y == y as f32 && v.position.z == z as f32) {
            voxel.voxel_type = voxel_type;
        } else if voxel_type != VoxelType::Empty {
            world.voxels.push(Voxel { position: Vector3::new(x as f32, y as f32, z as f32), voxel_type });
        }
    }
} 