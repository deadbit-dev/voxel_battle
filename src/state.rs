use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoxelType {
    Empty,
    Ground,
    Wall,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voxel {
    pub position: Vector3,
    pub voxel_type: VoxelType,
}

#[derive(Debug, Clone)]
pub struct World {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub voxels: Vec<Voxel>,
    pub voxel_size: f32,
}

impl Default for World {
    fn default() -> Self {
        Self {
            width: 25,
            height: 25,
            depth: 25,
            voxels: Vec::new(),
            voxel_size: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    pub position: Vector3,
    pub size: Vector3, // Size components for each axis (x, y, z)
    pub color: Color,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Vector3 { x: 25.0, y: 1.0, z: 25.0 },
            size: Vector3 { x: 0.5, y: 1.0, z: 0.5 }, // Half size in width and depth, full height
            color: Color { r: 255, g: 255, b: 255, a: 255 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: HashMap<i32, PlayerState>,
    pub world: World,
    pub screen_width: i32,
    pub screen_height: i32,
    pub player_inputs: PlayerInputs,
    pub next_player_id: i32,
    pub camera: Camera3D,
    pub camera_offset: Vector3,
    pub camera_smoothing: f32,
    pub camera_height: f32,
    pub camera_angle: f32,
    pub debug_mode: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            world: World::default(),
            screen_width: 800,
            screen_height: 600,
            player_inputs: HashMap::new(),
            next_player_id: 0,
            camera: Camera3D::perspective(
                Vector3::new(25.0, 25.0, 25.0),
                Vector3::new(12.5, 0.0, 12.5),
                Vector3::new(0.0, 1.0, 0.0),
                60.0,
            ),
            camera_offset: Vector3::new(0.0, 0.0, 15.0),
            camera_smoothing: 0.2,
            camera_height: 25.0,
            camera_angle: 45.0,
            debug_mode: false,
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerInput {
    pub movement: Vector2,
    pub movement_speed: f32,
}

// Define a type alias for player inputs
pub type PlayerInputs = HashMap<i32, PlayerInput>;