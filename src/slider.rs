use super::*;

use geng::ui::*;

pub struct Slider<'a> {
    cx: &'a Controller,
    sense: &'a mut Sense,
    pos: &'a mut Option<AABB<f64>>,
    value: f64,
    range: RangeInclusive<f64>,
    change: RefCell<&'a mut Option<f64>>,
}

impl<'a> Slider<'a> {
    const ANIMATION_SPEED: f32 = 5.0;

    pub fn new(cx: &'a Controller, text: String, value: f64, range: RangeInclusive<f64>) -> Self {
        Slider {
            cx,
            sense: cx.get_state(),
            pos: cx.get_state(),
            value,
            range,
            change: RefCell::new(cx.get_state()),
        }
    }

    pub fn get_change(&self) -> Option<f64> {
        self.change.borrow_mut().take()
    }
}

impl<'a> Widget for Slider<'a> {
    fn sense(&mut self) -> Option<&mut Sense> {
        Some(self.sense)
    }
    fn update(&mut self, delta_time: f64) {}
    fn draw(&mut self, cx: &mut DrawContext) {
        // cx.geng.draw_2d(cx.framebuffer, &geng::PixelPerfectCamera, &draw_2d::Quad::new())
    }
    fn handle_event(&mut self, event: &geng::Event) {
        let aabb = match *self.pos {
            Some(pos) => pos,
            None => return,
        };
        if self.sense.is_captured() {
            if let geng::Event::MouseDown { position, .. }
            | geng::Event::MouseMove { position, .. } = &event
            {
                let position = position.x - aabb.x_min;
                let new_value = *self.range.start()
                    + ((position - aabb.height() / 6.0) / (aabb.width() - aabb.height() / 3.0))
                        .clamp(0.0, 1.0)
                        * (*self.range.end() - *self.range.start());
                **self.change.borrow_mut() = Some(new_value);
            }
        }
    }

    fn calc_constraints(&mut self, _children: &ConstraintsContext) -> Constraints {
        Constraints {
            min_size: vec2(1.0, 1.0) * self.cx.theme().text_size as f64,
            flex: vec2(1.0, 0.0),
        }
    }
}
