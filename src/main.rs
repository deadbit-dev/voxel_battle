use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;
    
fn main() {
    let (mut rl,thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Game")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        
        d.clear_background(Color::BLACK);

        const TEXT: &str = "Wellcome :)";
        const FONT_SIZE: i32 = 20;
        let text_width: i32 = d.measure_text(TEXT, FONT_SIZE);
        let text_x: i32 = (WINDOW_WIDTH / 2) - (text_width / 2);
        let text_y: i32 = WINDOW_HEIGHT / 2;
        d.draw_text(TEXT, text_x, text_y, FONT_SIZE, Color::WHITE);
    }
}
