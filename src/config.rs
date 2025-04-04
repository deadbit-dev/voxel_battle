use raylib::core::color::Color;

pub const SCREEN_WIDTH: i32 = 800;
pub const SCREEN_HEIGHT: i32 = 600;

pub const GLSL_VERSION: i32 = 330;

pub const VOXEL_SIZE: f32 = 1.0;
pub const DEBUG_COLOR: Color = Color { r: 0, g: 255, b: 0, a: 128 }; // Semi-transparent green
pub const GRID_COLOR: Color = Color { r: 0, g: 255, b: 0, a: 128 }; // Very transparent green for grid
pub const LIGHT_COLOR: Color = Color { r: 255, g: 255, b: 255, a: 255 }; // White light source

// Predefined list of colors
pub const PLAYER_COLORS: [Color; 5] = [
    Color { r: 44, g: 93, b: 55, a: 255 },   // #2C5D37
    Color { r: 227, g: 197, b: 21, a: 255 }, // #E3C515
    Color { r: 238, g: 81, b: 177, a: 255 }, // #EE51B1
    Color { r: 165, g: 156, b: 211, a: 255 }, // #A59CD3
    Color { r: 75, g: 45, b: 159, a: 255 },  // #4B2D9F
];

// FIXME: world voxel grid has some offset
// FIXME: hoverd voxel a litle bit worond because of the offset
// FIXME: worng bbox of player some times

/* 
REFACTORING review all code
- separate game and editor logic
- use two cameras for game and editor (use two states)
- separate abilities from player?
*/

// TODO: rotating player and go forward
// TODO: multiplayer base logic - server and client, connecting/disconnecting, sending data
// TODO: add some sound
