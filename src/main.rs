#![windows_subsystem = "windows"]

mod ui;
mod utils;

use druid::commands::CLOSE_WINDOW;
use druid::image::{open, ImageBuffer, Rgb};

use druid::widget::{Container, Image};
use druid::{
    AppLauncher, Data, Env, EventCtx, ImageBuf, Lens, LocalizedString, Rect, Screen, Size,
    WidgetExt, WindowConfig, WindowDesc, WindowHandle, WindowId,
};

use rfd::FileDialog;
use std::sync::{mpsc, Arc};
use std::thread;
use ui::controllers::DragController;
use ui::ui::{build_root_widget, ViewStatus};
use utils::image_drawing::draw_image;
use utils::image_utils::quantize;
use utils::save_coordinates::load_colors_pos;

#[derive(Clone, Data, Lens)]
struct AppState {
    #[data(eq)]
    palette: Arc<Vec<Rgb<u8>>>,
    #[data(eq)]
    current_image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    #[data(eq)]
    colors_pos: Vec<(f64, f64)>,
    #[data(eq)]
    sub_window: Option<WindowId>,
    dithering: bool,
    pixel_interval: f64,
    view_status: ViewStatus,
    start_pos: Option<(f64, f64)>,
    end_pos: Option<(f64, f64)>,
    x_offset: f64,
    y_offset: f64,
}

impl AppState {
    fn get_image(&self) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if let Some(selected_path) = FileDialog::new().pick_file() {
            let palette = Arc::clone(&self.palette);
            let dithering = self.dithering.clone();

            let pos1 = self.start_pos.unwrap();
            let pos2 = self.end_pos.unwrap();

            let size = Rect::from_points(pos1, pos2).size();
            let (tx, rx) = mpsc::channel();

            // Spawn a new thread to open the image
            thread::spawn(move || match open(&selected_path) {
                Ok(image) => {
                    let result = quantize(
                        &image,
                        &palette,
                        (size.width as u32, size.height as u32),
                        dithering,
                    );
                    tx.send(result).unwrap();
                }
                Err(e) => panic!("Failed proccesing image: {}", e),
            });

            // Получаем размеры изображения из потока
            match rx.recv() {
                Ok(image) => return Some(image),
                Err(e) => panic!("Failed to get image: {}", e),
            }
        } else {
            None
        }
    }

    fn save_image(&mut self, ctx: &mut EventCtx, data: &AppState, env: &Env) {
        self.current_image = self.get_image().unwrap_or_default();

        self.show_image(ctx, data, env)
    }

    fn show_image(&mut self, ctx: &mut EventCtx, data: &AppState, env: &Env) {
        let widget = Container::new(Image::new(ImageBuf::from_raw(
            self.current_image.clone().into_raw(),
            druid::piet::ImageFormat::Rgb,
            self.current_image.width() as usize,
            self.current_image.height() as usize,
        )));
        let size = Size::new(
            self.current_image.width() as f64,
            self.current_image.height() as f64,
        );

        if self.sub_window.is_some() {
            ctx.submit_command(CLOSE_WINDOW.to(self.sub_window.unwrap()));
        }

        self.sub_window = Some(
            ctx.new_sub_window(
                WindowConfig::default()
                    .show_titlebar(false)
                    .window_size(size),
                widget.controller(DragController),
                data.clone(),
                env.clone(),
            ),
        );
    }

    fn switch_view_to_overlay(&self, window: &WindowHandle) {
        let display_rect = Screen::get_display_rect();
        let display_size = display_rect.size();

        window.set_size(display_size);
        window.set_always_on_top(true);
        window.show_titlebar(false);
        window.set_position(display_rect.origin());
    }

    fn switch_view_to_default(&self, window: &WindowHandle) {
        window.set_always_on_top(false);
        window.show_titlebar(true);
        window.set_size((300.0, 300.0));
        window.set_position(Screen::get_display_rect().center());
    }

    fn get_area(&mut self, ctx: &mut EventCtx) {
        self.switch_view_to_overlay(ctx.window());
        self.view_status = ViewStatus::Area;
    }

    fn get_palette(&mut self, ctx: &mut EventCtx) {
        self.switch_view_to_overlay(ctx.window());
        self.colors_pos = Vec::with_capacity(18);
        self.view_status = ViewStatus::Palette;
    }

    fn draw_image(&self) {
        let palette = Arc::clone(&self.palette);
        let colors_coordinates = (*palette)
            .clone()
            .into_iter()
            .zip(self.colors_pos.clone().into_iter())
            // .map(|(color, (x, y))| (color, (x as u32, y as u32)))
            .collect();

        draw_image(
            &self.current_image,
            &colors_coordinates,
            &self.start_pos.unwrap(),
            self.pixel_interval as u8,
        );
    }
}

fn main() {
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;

    let monitors = Screen::get_monitors();

    for monitor in &monitors {
        if monitor.is_primary() {
            let work_rect = monitor.virtual_work_rect();

            x_offset = work_rect.x0;
            y_offset = work_rect.y0;
        }
    }

    // Загружаем позиции цветов из файла
    let loaded_colors_pos = load_colors_pos("colors_pos.txt").unwrap_or(vec![]).clone();

    let initial_state = AppState {
        palette: Arc::new(vec![
            Rgb([0, 0, 0]),
            Rgb([102, 102, 102]),
            Rgb([0, 80, 205]),
            Rgb([255, 255, 255]),
            Rgb([170, 170, 170]),
            Rgb([38, 201, 255]),
            Rgb([1, 116, 32]),
            Rgb([153, 0, 0]),
            Rgb([150, 65, 18]),
            Rgb([17, 176, 60]),
            Rgb([255, 0, 19]),
            Rgb([255, 120, 41]),
            Rgb([176, 112, 28]),
            Rgb([153, 0, 78]),
            Rgb([203, 90, 87]),
            Rgb([255, 193, 38]),
            Rgb([255, 0, 143]),
            Rgb([254, 175, 168]),
        ]),
        colors_pos: loaded_colors_pos,
        current_image: ImageBuffer::new(1, 1),
        sub_window: None,
        dithering: true,
        pixel_interval: 2.0,
        view_status: ViewStatus::Default,
        start_pos: None,
        end_pos: None,
        x_offset,
        y_offset,
    };

    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("Drawing Bot"))
        .transparent(true)
        .window_size((300.0, 300.0));

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}
