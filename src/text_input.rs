use super::*;

use geng::ui::*;

pub struct TextInput<'a> {
    cx: &'a Controller,
    sense: &'a mut Sense,
    pos: &'a mut Option<AABB<f64>>,
    text: &'a mut String,
    t: &'a mut f64,
    max_len: usize,
    pub capture: &'a mut bool,
}

impl<'a> TextInput<'a> {
    pub fn new(cx: &'a Controller, text: &'a mut String, max_len: usize) -> Self {
        TextInput {
            cx,
            t: cx.get_state(),
            sense: cx.get_state(),
            pos: cx.get_state(),
            capture: cx.get_state(),
            text,
            max_len,
        }
    }
}

impl<'a> Widget for TextInput<'a> {
    fn sense(&mut self) -> Option<&mut Sense> {
        Some(self.sense)
    }
    fn update(&mut self, delta_time: f64) {
        *self.t += delta_time;
        if *self.t > 1.0 {
            *self.t = 0.0;
        }
    }
    fn draw(&mut self, cx: &mut DrawContext) {
        let font = cx.geng.default_font();
        let mut text = self.text.as_str();
        if text.is_empty() {
            if *self.capture {
                text = "";
            } else {
                text = "click to change your name";
            }
        }
        let _size = partial_min(
            cx.position.height() as f32,
            1.0 * cx.position.width() as f32
                / font.measure(text, 1.0).map_or(0.0, |aabb| aabb.width()),
        );
        let size = cx.position.height() as f32;
        let color = if *self.capture || self.sense.is_hovered() {
            cx.theme.hover_color
        } else {
            cx.theme.usable_color
        };
        let w = font.measure(text, size).map_or(0.0, |aabb| aabb.width());
        let text = if *self.t < 0.5 || !*self.capture {
            text.to_owned()
        } else {
            format!("{text}_")
        };
        font.draw(
            cx.framebuffer,
            &geng::PixelPerfectCamera,
            &text,
            cx.position.center().map(|x| x as f32)
                + vec2(-w / 2.0, -size * 0.5 - font.descender() * size),
            geng::TextAlign::LEFT,
            size,
            color,
        );
    }
    fn handle_event(&mut self, event: &geng::Event) {
        if let geng::Event::MouseDown { .. } = event {
            *self.capture = false;
        }
        if self.sense.take_clicked() {
            *self.capture = true;
        }
        // LOL
    }

    fn calc_constraints(&mut self, _children: &ConstraintsContext) -> Constraints {
        Constraints {
            min_size: vec2(300.0, self.cx.theme().text_size as f64),
            flex: vec2(0.0, 0.0),
        }
    }
}
