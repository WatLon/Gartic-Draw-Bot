use std::{collections::HashMap, thread};

use druid::image::{ImageBuffer, Pixel, Rgb};
use enigo::{
    Button,
    Coordinate::Abs,
    Direction::{Click, Press, Release},
    Enigo, Mouse, Settings,
};

struct DrawingBot {
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    start_position: (u32, u32),
    colors_coordinates: HashMap<Rgb<u8>, (u32, u32)>,
    pixels_lines_to_draw: HashMap<Rgb<u8>, Vec<((u32, u32), (u32, u32))>>,
    enigo: Enigo,
}

impl DrawingBot {
    fn new(
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
        colors_coordinates: HashMap<Rgb<u8>, (u32, u32)>,
        start_position: (u32, u32),
        pixels_interval: u8,
    ) -> Self {
        let dummy_bot = Self {
            image: image.clone(),
            start_position,
            colors_coordinates: colors_coordinates.clone(),
            pixels_lines_to_draw: HashMap::new(),
            enigo: Enigo::new(&Settings::default()).unwrap(),
        };

        let pixels_lines_to_draw = dummy_bot.extract_pixel_lines_to_draw(pixels_interval);

        Self {
            image,
            start_position,
            colors_coordinates: colors_coordinates,
            pixels_lines_to_draw: pixels_lines_to_draw,
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }

    fn extract_pixel_lines_to_draw(
        &self,
        pixels_interval: u8,
    ) -> HashMap<Rgb<u8>, Vec<((u32, u32), (u32, u32))>> {
        let (draw_vertically_lines, nb_vertical_lines) =
            self.extract_lines_to_draw(true, pixels_interval);
        let (draw_horizontally_lines, nb_horizontal_lines) =
            self.extract_lines_to_draw(false, pixels_interval);

        if nb_vertical_lines > nb_horizontal_lines {
            draw_horizontally_lines
        } else {
            draw_vertically_lines
        }
    }

    fn extract_lines_to_draw(
        &self,
        vertically: bool,
        pixels_interval: u8,
    ) -> (HashMap<Rgb<u8>, Vec<((u32, u32), (u32, u32))>>, u32) {
        let (width, height) = self.image.dimensions();
        let (w, h) = if vertically {
            (width, height)
        } else {
            (height, width)
        };

        let mut lines: HashMap<Rgb<u8>, Vec<((u32, u32), (u32, u32))>> = HashMap::new();
        let mut nb_lines = 0;

        for x in (0..w).step_by(pixels_interval as usize) {
            let mut line_color: Option<Rgb<u8>> = None;
            let mut line_start: (u32, u32) = (0, 0);
            let mut line_end: (u32, u32) = (0, 0);

            for y in (0..h).step_by(pixels_interval as usize) {
                let (pixel, current_position) = if vertically {
                    let pixel = self.image.get_pixel(x, y).to_rgb();
                    (
                        pixel,
                        (self.start_position.0 + x, self.start_position.1 + y),
                    )
                } else {
                    let pixel = self.image.get_pixel(y, x).to_rgb();
                    (
                        pixel,
                        (self.start_position.0 + y, self.start_position.1 + x),
                    )
                };

                if line_color.is_none() {
                    line_color = Some(pixel);
                    line_start = current_position;
                } else if let Some(lc) = line_color {
                    if lc != pixel {
                        if lc != Rgb([255, 255, 255]) {
                            nb_lines += 1;
                        }
                        lines
                            .entry(lc)
                            .or_insert_with(Vec::new)
                            .push((line_start, line_end));

                        line_color = Some(pixel);
                        line_start = current_position;
                    }
                }
                line_end = current_position;
            }

            if let Some(lc) = line_color {
                if lc != Rgb([255, 255, 255]) {
                    nb_lines += 1;
                }
                lines
                    .entry(lc)
                    .or_insert_with(Vec::new)
                    .push((line_start, line_end));
            }
        }

        (lines, nb_lines)
    }

    fn change_color(&mut self, color: Rgb<u8>) {
        let coordinate = self.colors_coordinates.get(&color).unwrap();
        self.enigo
            .move_mouse(coordinate.0 as i32, coordinate.1 as i32, Abs)
            .unwrap();
        self.enigo.button(Button::Left, Click).unwrap();
    }

    fn draw_line(&mut self, line: ((u32, u32), (u32, u32))) {
        self.enigo
            .move_mouse(line.0 .0 as i32, line.0 .1 as i32, Abs)
            .unwrap();
        self.enigo.button(Button::Left, Press).unwrap();

        self.enigo
            .move_mouse(line.1 .0 as i32, line.1 .1 as i32, Abs)
            .unwrap();
        self.enigo.button(Button::Left, Release).unwrap();
    }
    fn draw(&mut self) {
        for (color, lines) in self.pixels_lines_to_draw.clone() {
            if color != Rgb([255, 255, 255]) {
                self.change_color(color);
                for line in lines {
                    self.draw_line(line);
                    thread::sleep(std::time::Duration::from_millis(15));
                }
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub fn draw_image(
    image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    colors_coordinates: &HashMap<Rgb<u8>, (u32, u32)>,
    start_pos: &(f64, f64),
    pixel_interval: u8,
) {
    let mut bot = DrawingBot::new(
        image.clone(),
        colors_coordinates.clone(),
        (start_pos.0 as u32, start_pos.1 as u32),
        pixel_interval,
    );
    bot.draw();
}
