use raylib::prelude::*;
use rand::Rng;

// Function to generate a random color from the predefined list
pub fn generate_random_color(excluded_colors: &[Color], list_colors: &[Color]) -> Option<Color> {
    // Find available colors
    let available_colors: Vec<Color> = list_colors.iter()
        .filter(|&&color| !excluded_colors.contains(&color))
        .cloned()
        .collect();
    
    if available_colors.is_empty() {
        None
    } else {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..available_colors.len());
        Some(available_colors[index])
    }
}

// Linear interpolation for f32
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}