use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread, time,
};

use rdev::{self, simulate};

use druid::image::{ImageBuffer, Pixel, Rgb};

struct DrawingBot {
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    start_position: (f64, f64),
    colors_coordinates: HashMap<Rgb<u8>, (f64, f64)>,
    pixels_lines_to_draw: HashMap<Rgb<u8>, Vec<((f64, f64), (f64, f64))>>,
    drawing: Arc<Mutex<bool>>,
}

impl DrawingBot {
    fn new(
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
        colors_coordinates: HashMap<Rgb<u8>, (f64, f64)>,
        start_position: (f64, f64),
        pixels_interval: u8,
    ) -> Self {
        let dummy_bot = Self {
            image: image.clone(),
            start_position,
            colors_coordinates: colors_coordinates.clone(),
            pixels_lines_to_draw: HashMap::new(),
            drawing: Arc::new(Mutex::new(true)),
        };

        let pixels_lines_to_draw = dummy_bot.extract_pixel_lines_to_draw(pixels_interval);

        Self {
            image,
            start_position,
            colors_coordinates: colors_coordinates,
            pixels_lines_to_draw: pixels_lines_to_draw,
            drawing: Arc::new(Mutex::new(true)),
        }
    }

    fn extract_pixel_lines_to_draw(
        &self,
        pixels_interval: u8,
    ) -> HashMap<Rgb<u8>, Vec<((f64, f64), (f64, f64))>> {
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
    ) -> (HashMap<Rgb<u8>, Vec<((f64, f64), (f64, f64))>>, i32) {
        let (width, height) = self.image.dimensions();
        let (w, h) = if vertically {
            (width, height)
        } else {
            (height, width)
        };

        let mut lines: HashMap<Rgb<u8>, Vec<((f64, f64), (f64, f64))>> = HashMap::new();
        let mut nb_lines = 0;

        for x in (0..w).step_by(pixels_interval as usize) {
            let mut line_color: Option<Rgb<u8>> = None;
            let mut line_start: (f64, f64) = (0.0, 0.0);
            let mut line_end: (f64, f64) = (0.0, 0.0);

            for y in (0..h).step_by(pixels_interval as usize) {
                let (pixel, current_position) = if vertically {
                    let pixel = self.image.get_pixel(x, y).to_rgb();
                    (
                        pixel,
                        (
                            self.start_position.0 + x as f64,
                            self.start_position.1 + y as f64,
                        ),
                    )
                } else {
                    let pixel = self.image.get_pixel(y, x).to_rgb();
                    (
                        pixel,
                        (
                            self.start_position.0 + y as f64,
                            self.start_position.1 + x as f64,
                        ),
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

    fn change_color(&self, color: Rgb<u8>) {
        let coordinate = self.colors_coordinates.get(&color).unwrap();
        self.mouse_simulation(&rdev::EventType::MouseMove {
            x: coordinate.0,
            y: coordinate.1,
        });
        self.mouse_simulation(&rdev::EventType::ButtonPress(rdev::Button::Left));
        self.mouse_simulation(&rdev::EventType::ButtonRelease(rdev::Button::Left))
        // self.enigo
        //     .move_mouse(coordinate.0 as i32, coordinate.1 as i32, Abs)
        //     .unwrap();
        // self.enigo.button(Button::Left, Click).unwrap();
    }

    fn draw_line(&mut self, line: ((f64, f64), (f64, f64))) {
        self.mouse_simulation(&rdev::EventType::MouseMove {
            x: line.0 .0,
            y: line.0 .1,
        });
        self.mouse_simulation(&rdev::EventType::ButtonPress(rdev::Button::Left));

        self.mouse_simulation(&rdev::EventType::MouseMove {
            x: line.1 .0,
            y: line.1 .1,
        });
        self.mouse_simulation(&rdev::EventType::ButtonRelease(rdev::Button::Left));
        // self.enigo
        //     .move_mouse(line.0 .0 as i32, line.0 .1 as i32, Abs)
        //     .unwrap();
        // self.enigo.button(Button::Left, Press).unwrap();

        // self.enigo
        //     .move_mouse(line.1 .0 as i32, line.1 .1 as i32, Abs)
        //     .unwrap();
        // self.enigo.button(Button::Left, Release).unwrap();
    }

    fn stop_drawing(&self) {
        let drawing = Arc::clone(&self.drawing);
        thread::spawn(move || {
            let callback = move |event: rdev::Event| match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    if key == rdev::Key::Escape {
                        let mut is_drawing = drawing.lock().unwrap();
                        *is_drawing = false;
                    }
                }
                _ => {}
            };

            if let Err(e) = rdev::listen(callback) {
                eprintln!("Error listening for events: {:?}", e);
            }
        });
    }
    fn draw(&mut self) {
        let _ = &self.stop_drawing();
        for (color, lines) in self.pixels_lines_to_draw.clone() {
            if color != Rgb([255, 255, 255]) {
                self.change_color(color);
                for line in lines {
                    if !*self.drawing.lock().unwrap() {
                        break;
                    }
                    self.draw_line(line);
                    thread::sleep(time::Duration::from_millis(10))
                }
            }
        }
    }

    fn mouse_simulation(&self, event_type: &rdev::EventType) {
        simulate(event_type).unwrap();
    }
}

pub fn draw_image(
    image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    colors_coordinates: &HashMap<Rgb<u8>, (f64, f64)>,
    start_pos: &(f64, f64),
    pixel_interval: u8,
) {
    let mut bot = DrawingBot::new(
        image.clone(),
        colors_coordinates.clone(),
        (start_pos.0, start_pos.1),
        pixel_interval,
    );
    bot.draw();
}
