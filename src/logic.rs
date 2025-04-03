use raylib::prelude::*;
use crate::state::{GameState, World, VoxelType, PlayerInput, Voxel, PlayerState};

pub fn init(state: &mut GameState) {
    state.world = World::default();
    state.next_player_id = 1; // Start from 1 for gamepad players
    
    // Create floor voxels
    for x in 0..state.world.width {
        for z in 0..state.world.depth {
            set_voxel(&mut state.world, x, 0, z, VoxelType::Ground);
        }
    }
    
    state.player_inputs.insert(0, PlayerInput {
        movement: Vector2::zero(),
        movement_speed: 5.0, // 5 units per second
    });
    
    // Set player position above the floor
    let mut player = PlayerState::default();
    player.position.x = state.world.width as f32 / 2.0; // Center X
    player.position.y = 1.0; // Set player 1 unit above the floor
    player.position.z = state.world.depth as f32 / 2.0; // Center Z
    state.players.insert(0, player);
}

pub fn update(state: &mut GameState, delta: f32) {
    // Toggle debug mode with M
    unsafe {
        if ffi::IsKeyPressed(KeyboardKey::KEY_M as i32) {
            state.debug_mode = !state.debug_mode;
        }
    }

    update_player_inputs(state);
    update_player_position(state, delta);
    handle_voxel_input(state);
    update_camera(state, delta);
}

pub fn update_player_inputs(state: &mut GameState) {
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
                    
                    // Increase movement speed for gamepad players
                    input.movement_speed = 8.0;
                    input.movement = movement;
                }
            }
        }
    }

    // Check for new gamepads only if we haven't reached the maximum number of players
    if state.next_player_id < 5 { // Allow up to 4 gamepad players (IDs 1-4)
        for i in 0..4 {
            unsafe {
                let is_available = ffi::IsGamepadAvailable(i);
                if is_available {
                    let player_id = i + 1; // Gamepad 0 -> Player 1, Gamepad 1 -> Player 2, etc.
                    if !state.player_inputs.contains_key(&player_id) {
                        println!("New gamepad detected: {} -> Player {}", i, player_id);
                        
                        // Create player input with higher speed
                        state.player_inputs.insert(player_id, PlayerInput {
                            movement: Vector2::zero(),
                            movement_speed: 8.0, // Increased speed for gamepad players
                        });
                        
                        // Create player state
                        let mut player = PlayerState::default();
                        player.position.x = state.world.width as f32 / 2.0; // Center X
                        player.position.y = 1.0; // Set player 1 unit above the floor
                        player.position.z = state.world.depth as f32 / 2.0; // Center Z
                        player.color = Color { r: 255, g: 0, b: 0, a: 255 }; // Red color for gamepad players
                        state.players.insert(player_id, player);
                        
                        state.next_player_id = player_id + 1;
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

pub fn update_player_position(state: &mut GameState, delta: f32) {
    for (id, input) in state.player_inputs.iter() {
        if let Some(player) = state.players.get_mut(id) {
            // Calculate movement vector
            let movement = Vector3::new(
                input.movement.x * input.movement_speed * delta,
                0.0,
                input.movement.y * input.movement_speed * delta
            );

            if movement.length() > 0.0 {
                let new_position = player.position + movement;
                let collision = check_voxel_collision(&state.world, new_position, player.size);
                if collision.is_none() {
                    player.position = new_position;
                }
                // TODO: implement more precise collision detection
                // else {
                //     println!("Collision: {:?}", collision.unwrap());
                //     let mut voxel_pos = collision.unwrap().position;
                //     let normalized_movement = movement.normalized();
                //     voxel_pos.x -= normalized_movement.x;
                //     voxel_pos.z -= normalized_movement.z;
                //     let voxel_size = state.world.voxel_size;
                //     let mut voxel_x = voxel_pos.x * voxel_size;
                //     let mut voxel_z = voxel_pos.z * voxel_size;
                //     if normalized_movement.x > 0.0 {
                //         voxel_x += voxel_size / 2.0 - player.size.x / 2.0;
                //     } else {
                //         voxel_x -= voxel_size / 2.0 + player.size.x / 2.0;
                //     }
                //     if normalized_movement.z > 0.0 {
                //         voxel_z += voxel_size / 2.0 - player.size.z / 2.0;
                //     } else {
                //         voxel_z -= voxel_size / 2.0 + player.size.z / 2.0;
                //     }
                //     println!("Voxel: {:?}, {:?}, {:?}", voxel_pos.x, voxel_pos.y, voxel_pos.z);
                //     player.position = Vector3::new(voxel_x as f32, player.position.y, voxel_z as f32);
                // }
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
    let ray: Ray = unsafe { ffi::GetScreenToWorldRay(mouse_pos, state.camera.into()).into() };
    
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

    if let Some((x, y, z, _, normal)) = closest_collision {
        // Check if there's already a voxel at this position
        let existing_voxel = get_voxel(&state.world, x, y, z);
        
        if unsafe { ffi::IsMouseButtonPressed(MouseButton::MOUSE_BUTTON_LEFT as i32) } {
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
        }
        if unsafe { ffi::IsMouseButtonPressed(MouseButton::MOUSE_BUTTON_RIGHT as i32) } {
            // Only remove if there's a voxel at this position and it's not occupied by a player
            if existing_voxel != VoxelType::Empty && !is_voxel_occupied_by_player(state, x, y, z) {
                set_voxel(&mut state.world, x, y, z, VoxelType::Empty);
            }
        }
    }
}

pub fn get_voxel(world: &World, x: i32, y: i32, z: i32) -> VoxelType {
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

pub fn set_voxel(world: &mut World, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
    if is_valid_position(world, x, y, z) {
        if let Some(voxel) = world.voxels.iter_mut().find(|v| v.position.x == x as f32 && v.position.y == y as f32 && v.position.z == z as f32) {
            voxel.voxel_type = voxel_type;
        } else if voxel_type != VoxelType::Empty {
            world.voxels.push(Voxel { position: Vector3::new(x as f32, y as f32, z as f32), voxel_type });
        }
    }
}

fn is_valid_position(world: &World, x: i32, y: i32, z: i32) -> bool {
    x >= 0 && x < world.width && y >= 0 && y < world.height && z >= 0 && z < world.depth
}

fn update_camera(state: &mut GameState, delta: f32) {
    if let Some(player) = state.players.get(&0) { // Follow first player
        // Calculate target camera position with fixed height and angle
        let angle_rad = state.camera_angle.to_radians();
        
        // Calculate target position behind player
        let target_x = player.position.x + state.camera_offset.z * angle_rad.sin();
        let target_z = player.position.z + state.camera_offset.z * angle_rad.cos();
        
        // Create target position vector
        let target_position = Vector3::new(target_x, state.camera_height, target_z);
        
        // Smoothly move camera towards target position
        state.camera.position = state.camera.position.lerp(
            target_position,
            state.camera_smoothing * delta * 10.0
        );
        
        // Update camera target to look at player
        state.camera.target = player.position;
    }
}