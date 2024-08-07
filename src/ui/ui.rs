use druid::piet::{FontFamily, Text, TextLayout, TextLayoutBuilder};
use druid::widget::{
    Button, Checkbox, Container, Flex, Label, Painter, SizedBox, Slider, ViewSwitcher,
};
use druid::{Color, Data, Env, RenderContext, Screen, Widget, WidgetExt};

use crate::AppState;

use super::controllers::PainterController;

#[derive(Copy, Clone, Data, PartialEq)]
pub enum ViewStatus {
    Default,
    Area,
    Palette,
}

pub enum Mode {
    Area,
    Palette,
}

pub fn build_root_widget() -> impl Widget<AppState> {
    let view_switcher = ViewSwitcher::new(
        |data: &AppState, _env| data.view_status,
        |_ctx, data, _env| match data.view_status {
            ViewStatus::Default => Box::new(
                Flex::column()
                    .with_child(Button::new("Select area").on_click(
                        |ctx, data: &mut AppState, _env| {
                            // ctx.window().set_size((1920.0, 1080.0));
                            data.get_area(ctx);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Load Image").on_click(
                        |_ctx, data: &mut AppState, _env| {
                            data.save_image(_ctx, &data.clone(), _env);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Select palette").on_click(
                        |ctx, data: &mut AppState, _env| {
                            data.get_palette(ctx);
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Button::new("Draw image").on_click(
                        |_ctx, data: &mut AppState, _env| {
                            data.draw_image();
                        },
                    ))
                    .with_spacer(10.0)
                    .with_child(Checkbox::new("Dither").lens(AppState::dithering))
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
