use druid::{widget::Controller, EventCtx, Widget};
use druid::{Env, Event};

use crate::utils::save_coordinates::save_colors_pos;
use crate::AppState;

use super::ui::{Mode, ViewStatus};

pub struct PainterController {
    mode: Mode,
}

impl PainterController {
    pub fn new(mode: Mode) -> PainterController {
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
                        data.switch_view_to_default(ctx.window());
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
                        data.switch_view_to_default(ctx.window());
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

pub struct DragController;

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
