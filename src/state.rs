use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoxelType {
    Empty,
    Ground,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voxel {
    pub position: Vector3,
    pub voxel_type: VoxelType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Lighting,
    Outline,
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
    pub original_color: Color, // Store original color for dash effect
    pub velocity: Vector3, // Current velocity
    pub acceleration: Vector3, // Current acceleration
    pub is_dashing: bool, // Whether the player is currently dashing
    pub dash_cooldown: f32, // Time remaining before next dash
    pub dash_direction: Vector3, // Direction of the current dash
    pub pre_dash_velocity: Vector3, // Velocity before starting a dash
    pub is_ready: bool, // Whether the player is ready to spawn
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Vector3 { x: 25.0, y: 1.0, z: 25.0 },
            size: Vector3 { x: 0.5, y: 1.0, z: 0.5 }, // Half size in width and depth, full height
            color: Color { r: 150, g: 150, b: 150, a: 255 }, // Darker gray color
            original_color: Color { r: 150, g: 150, b: 150, a: 255 }, // Same darker gray color
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            is_dashing: false,
            dash_cooldown: 0.0,
            dash_direction: Vector3::zero(),
            pre_dash_velocity: Vector3::zero(),
            is_ready: false, // Players start not ready
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightSource {
    pub position: Vector3,
    pub target: Vector3,
    pub color: Color,
    pub enabled: bool,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            position: Vector3::new(25.0, 10.0, 0.0), // Side of the map, at medium height
            target: Vector3::new(12.5, 0.0, 12.5),  // Center of the map
            color: Color { r: 255, g: 220, b: 180, a: 255 }, // Warm white light
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorCameraState {
    pub rotation: Vector2,
    pub distance: f32,
    pub center: Vector3,
    pub game_camera: Option<Camera3D>,
    pub game_camera_offset: Option<Vector3>,
    pub game_camera_height: Option<f32>,
    pub game_camera_angle: Option<f32>,
}

impl Default for EditorCameraState {
    fn default() -> Self {
        Self {
            rotation: Vector2::new(0.0, 1.0),
            distance: 25.0,
            center: Vector3::zero(),
            game_camera: None,
            game_camera_offset: None,
            game_camera_height: None,
            game_camera_angle: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub active: bool,
    pub camera: EditorCameraState,
    pub hovered_voxel: Option<(i32, i32, i32)>,
    pub build_mode: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            camera: EditorCameraState::default(),
            active: false,
            hovered_voxel: None,
            build_mode: true, // Start in build mode
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraState {
    pub camera: Camera3D,
    pub offset: Vector3,
    pub height: f32,
    pub angle: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            camera: Camera3D::perspective(
                Vector3::new(25.0, 25.0, 25.0),
                Vector3::new(12.5, 0.0, 12.5),
                Vector3::new(0.0, 1.0, 0.0),
                60.0,
            ),
            offset: Vector3::new(0.0, 0.0, 15.0),
            height: 25.0,
            angle: 45.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: HashMap<i32, PlayerState>,
    pub world: World,
    pub player_inputs: PlayerInputs,
    pub next_player_id: i32,
    pub camera_state: CameraState,
    pub editor: EditorState,
    pub shaders: HashMap<ShaderType, ffi::Shader>,
    pub light_source: LightSource,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            world: World::default(),
            player_inputs: HashMap::new(),
            next_player_id: 0,
            camera_state: CameraState::default(),
            editor: EditorState::default(),
            shaders: HashMap::new(),
            light_source: LightSource::default(),
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