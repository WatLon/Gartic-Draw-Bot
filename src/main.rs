#![windows_subsystem = "windows"]

mod utils;

use druid::commands::CLOSE_WINDOW;
use druid::image::{open, ImageBuffer, Rgb};
use druid::piet::{FontFamily, Text, TextLayout, TextLayoutBuilder};
use druid::widget::{
    Button, Checkbox, Container, Controller, Flex, Image, Label, Painter, SizedBox, Slider,
    ViewSwitcher,
};
use druid::{
    AppLauncher, Color, Data, Env, Event, EventCtx, ImageBuf, Lens, LocalizedString, Rect,
    RenderContext, Screen, Size, Widget, WidgetExt, WindowConfig, WindowDesc, WindowId,
};
use rfd::FileDialog;
use std::sync::{mpsc, Arc};
use std::thread;
use utils::image_drawing::draw_image;
use utils::image_utils::quantize;
use utils::save_coordinates::{load_colors_pos, save_colors_pos};

#[derive(Clone, Copy, Data, PartialEq)]
enum ViewStatus {
    Default,
    Area,
    Palette,
}

enum Mode {
    Area,
    Palette,
}

struct PainterController {
    mode: Mode,
}

impl PainterController {
    fn new(mode: Mode) -> PainterController {
        PainterController { mode }
    }
}

impl<W: Widget<AppState>> Controller<AppState, W> for PainterController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match self.mode {
            Mode::Area => match event {
                Event::MouseDown(me) => {
                    if me.buttons.has_right() {
                        data.view_status = ViewStatus::Default;
                        data.switch_view_to_default(ctx);
                        return;
                    }
                    ctx.set_active(true);
                    let pos: (f64, f64) = ctx.to_screen(me.pos).into();
                    data.start_pos = Some(pos);
                }
                Event::MouseMove(me) if ctx.is_active() && me.buttons.has_left() => {
                    if data.start_pos.is_some() {
                        let pos: (f64, f64) = ctx.to_screen(me.pos).into();
                        data.end_pos = Some(pos);
                    }
                }
                Event::MouseUp(_me) if ctx.is_active() => ctx.set_active(false),
                _ => (),
            },
            Mode::Palette => match event {
                Event::MouseDown(me) => {
                    if me.buttons.has_right() || data.colors_pos.len() >= 18 {
                        data.view_status = ViewStatus::Default;
                        data.switch_view_to_default(ctx);
                        save_colors_pos("colors_pos.txt", &data.colors_pos).unwrap();
                        return;
                    }
                    ctx.set_active(true);
                    let pos: (f64, f64) = ctx.to_screen(me.pos).into();
                    data.colors_pos.push(pos);
                }
                _ => (),
            },
        }
        child.event(ctx, event, data, env)
    }
}

struct DragController;

impl<T, W: Widget<T>> Controller<T, W> for DragController {
    fn event(
        &mut self,
        _child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        _data: &mut T,
        _env: &Env,
    ) {
        if let Event::MouseMove(_) = event {
            ctx.window().handle_titlebar(true);
        }
    }
}

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
            thread::spawn(move || {
                if let Ok(image) = open(&selected_path) {
                    let result = quantize(
                        &image,
                        &palette,
                        (size.width as u32, size.height as u32),
                        dithering,
                    );

                    tx.send(result).unwrap();
                }
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
    fn switch_view_to_overlay(&self, ctx: &mut EventCtx) {
        let window = ctx.window();

        let display_rect = Screen::get_display_rect();
        let display_size = display_rect.size();

        window.show_titlebar(false);
        window.set_position(display_rect.origin());
        window.set_size(display_size);
        window.set_size(display_size);
        window.set_size(display_size);
        window.set_always_on_top(true);
        window.resizable(false);
    }

    fn switch_view_to_default(&self, ctx: &mut EventCtx) {
        let window = ctx.window();

        window.set_always_on_top(false);
        window.show_titlebar(true);
        window.set_size((300.0, 300.0));
        window.resizable(true);
        window.set_position(Screen::get_display_rect().center());
    }

    fn get_area(&mut self, ctx: &mut EventCtx) {
        self.switch_view_to_overlay(ctx);
        self.view_status = ViewStatus::Area;
    }

    fn get_palette(&mut self, ctx: &mut EventCtx) {
        self.switch_view_to_overlay(ctx);
        self.colors_pos = Vec::with_capacity(18);
        self.view_status = ViewStatus::Palette;
    }
}

fn build_root_widget() -> impl Widget<AppState> {
    let view_switcher = ViewSwitcher::new(
        |data: &AppState, _env| data.view_status,
        |_ctx, data, _env| match data.view_status {
            ViewStatus::Default => Box::new(
                Flex::column()
                    .with_child(Button::new("Choose area").on_click(
                        |ctx, data: &mut AppState, _env| {
                            data.get_area(ctx);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Choose Image").on_click(
                        |_ctx, data: &mut AppState, _env| {
                            data.save_image(_ctx, &data.clone(), _env);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Choose palette").on_click(
                        |ctx, data: &mut AppState, _env| {
                            data.get_palette(ctx);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Draw image").on_click(
                        |_ctx, data: &mut AppState, _env| {
                            let palette = Arc::clone(&data.palette);
                            let colors_coordinates = (*palette)
                                .clone()
                                .into_iter()
                                .zip(data.colors_pos.clone().into_iter())
                                .map(|(color, (x, y))| (color, (x as u32, y as u32)))
                                .collect();

                            draw_image(
                                &data.current_image,
                                &colors_coordinates,
                                &data.start_pos.unwrap(),
                                data.pixel_interval as u8,
                            );
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Checkbox::new("Dithering").lens(AppState::dithering))
                    .with_spacer(10.0)
                    .with_child(Label::new(|data: &AppState, _env: &Env| {
                        format!("Pixel interval: {}", data.pixel_interval)
                    }))
                    .with_spacer(3.0)
                    .with_child(
                        Slider::new()
                            .with_range(1.0, 5.0)
                            .with_step(1.0)
                            .lens(AppState::pixel_interval),
                    )
                    .with_flex_child(SizedBox::empty().expand(), 1.0)
                    .background(Color::GRAY)
                    .expand(),
            ),
            ViewStatus::Area => Box::new(
                Flex::column().with_flex_child(
                    Painter::new(|ctx, data: &AppState, _env| {
                        let bounds = ctx.size().to_rect();

                        ctx.fill(bounds, &Color::rgba8(0, 0, 0, 64));
                        ctx.stroke(Screen::get_display_rect(), &Color::BLUE, 1.0);
                        ctx.stroke(bounds, &Color::GREEN, 1.0);
                        if let Some(start_pos) = data.start_pos {
                            ctx.clear(bounds, Color::rgba8(0, 0, 0, 128));
                            if let Some(end_pos) = data.end_pos {
                                let rect = druid::Rect::new(
                                    start_pos.0 - data.x_offset,
                                    start_pos.1 - data.y_offset,
                                    end_pos.0 - data.x_offset,
                                    end_pos.1 - data.y_offset,
                                );
                                ctx.clear(rect, Color::TRANSPARENT);
                                ctx.stroke(rect, &Color::RED, 1.0);
                            }
                        }
                    })
                    .controller(PainterController::new(Mode::Area)),
                    10.0,
                ),
            ),
            ViewStatus::Palette => Box::new(
                Flex::column().with_flex_child(
                    Painter::new(|ctx, data: &AppState, _env| {
                        let bounds = ctx.size().to_rect();

                        ctx.fill(bounds, &Color::rgba8(0, 0, 0, 16));
                        for pos in &data.colors_pos {
                            let text = ctx.text();
                            let layout = text
                                .new_text_layout(format!(
                                    "{}",
                                    data.colors_pos.iter().position(|&p| p == *pos).unwrap() + 1
                                ))
                                // .new_text_layout(format!("{}x{}", pos.0, pos.1))
                                .font(FontFamily::SERIF, 24.0)
                                .text_color(Color::RED)
                                .build()
                                .unwrap();
                            let layout_size = layout.size();
                            ctx.draw_text(
                                &layout,
                                (
                                    pos.0 - layout_size.width / 2.0 - data.x_offset,
                                    pos.1 - layout_size.height / 2.0 - data.y_offset,
                                ),
                            );
                        }
                    })
                    .controller(PainterController::new(Mode::Palette)),
                    10.0,
                ),
            ),
        },
    );

    Container::new(view_switcher)
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
