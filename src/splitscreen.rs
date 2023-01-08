use super::*;

pub struct SplitScreen {
    geng: Geng,
    inner: Vec<geng::StateManager>,
    texture: ugli::Texture,
}

impl SplitScreen {
    pub fn new(geng: &Geng, list: impl IntoIterator<Item = Box<dyn geng::State>>) -> Self {
        Self {
            geng: geng.clone(),
            inner: list
                .into_iter()
                .map(|state| {
                    let mut sm = geng::StateManager::new();
                    sm.push(state);
                    sm
                })
                .collect(),
            texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
        }
    }
}

impl geng::State for SplitScreen {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let texture_size = vec2(
            framebuffer.size().x / self.inner.len(),
            framebuffer.size().y,
        );
        if texture_size != self.texture.size() {
            self.texture = ugli::Texture::new_uninitialized(self.geng.ugli(), texture_size);
        }
        let mut x = 0;
        for inner in &mut self.inner {
            {
                let mut framebuffer = ugli::Framebuffer::new_color(
                    self.geng.ugli(),
                    ugli::ColorAttachment::Texture(&mut self.texture),
                );
                inner.draw(&mut framebuffer);
            }
            self.geng.draw_2d(
                framebuffer,
                &geng::PixelPerfectCamera,
                &draw_2d::TexturedQuad::new(
                    AABB::point(vec2(x as f32, 0.0))
                        .extend_positive(texture_size.map(|x| x as f32)),
                    &self.texture,
                ),
            );
            x += texture_size.x;
        }
    }
    fn update(&mut self, delta_time: f64) {
        for inner in &mut self.inner {
            inner.update(delta_time);
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        let mut event = event;
        match &mut event {
            geng::Event::MouseDown { position, .. }
            | geng::Event::MouseUp { position, .. }
            | geng::Event::MouseMove { position, .. } => {
                for inner in &mut self.inner {
                    if position.x < self.texture.size().x as f64 {
                        inner.handle_event(event);
                        break;
                    }
                    position.x -= self.texture.size().x as f64;
                }
            }
            _ => {
                for inner in &mut self.inner {
                    inner.handle_event(event.clone());
                }
            }
        }
    }
}
