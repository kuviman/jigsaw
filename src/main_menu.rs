use super::*;

struct ConfigScreen {
    assets: Rc<Assets>,
    geng: Geng,
    config: RoomConfig,
    addr: String,
    transition: Option<geng::Transition>,
    texture: ugli::Texture,
}

impl ConfigScreen {
    fn new(geng: &Geng, assets: Rc<Assets>, addr: &str) -> Self {
        let texture = generate_background(geng, &assets);
        Self {
            assets,
            addr: addr.to_owned(),
            geng: geng.clone(),
            config: RoomConfig {
                seed: thread_rng().gen(),
                size: vec2(30, 1), // LUL
                image: 0,
            },
            transition: None,
            texture,
        }
    }
}

impl geng::State for ConfigScreen {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        let texture = &self.texture;
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let size = texture.size().map(|x| x as f32);
        let ratio = (framebuffer_size.y / size.y).max(framebuffer_size.x / size.x);
        let size = size * ratio;
        self.geng.draw_2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw_2d::TexturedQuad::new(
                AABB::point(framebuffer_size / 2.0).extend_symmetric(size * 0.5),
                texture,
            ),
        );
    }
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let play_button = Button::new(cx, "PLAY");
        if play_button.was_clicked() {
            let future = {
                let geng = self.geng.clone();
                let addr = self.addr.clone();
                let mut config = self.config.clone();
                config.size = (1..=config.size.x)
                    .filter_map(|x| {
                        if config.size.x % x == 0 {
                            Some(vec2(x, config.size.x / x))
                        } else {
                            None
                        }
                    })
                    .min_by_key(|size| {
                        let aspect = size.x as f64 / size.y as f64;
                        let image = &self.assets.images[config.image];
                        let image_aspect = image.size().x as f64 / image.size().y as f64;
                        r64(aspect - image_aspect).abs()
                    })
                    .unwrap();
                async move {
                    let mut con: Connection = geng::net::client::connect(&addr).await;
                    con.send(ClientMessage::CreateRoom(config));
                    let room = match con.next().await {
                        Some(ServerMessage::RoomCreated(name)) => name,
                        _ => unreachable!(),
                    };
                    info!("room: {:?}", room);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::window()
                        .unwrap()
                        .location()
                        .set_href(&format!("?room={}", room))
                        .unwrap();
                    game::run(&geng, &addr, &room, None)
                }
            };
            let state =
                geng::LoadingScreen::new(&self.geng, geng::EmptyLoadingScreen, future, |state| {
                    state
                });
            self.transition = Some(geng::Transition::Switch(Box::new(state)));
        }
        let image_button = Button::new(cx, &format!("Image: Harvest #{}", self.config.image + 1));
        if image_button.was_clicked() {
            self.config.image = (self.config.image + 1) % self.assets.images.len();
        }
        let difficulty_button =
            Button::new(cx, &format!("Difficulty: {} pieces", self.config.size.x));
        if difficulty_button.was_clicked() {
            let options = [30, 120, 500, 1000];
            self.config.size.x = options[(options
                .iter()
                .position(|x| *x == self.config.size.x)
                .unwrap()
                + 1)
                % options.len()];
        }
        (
            image_button.center(),
            difficulty_button.center(),
            play_button.center(),
        )
            .column()
            .center()
            .boxed()
    }
    fn transition(&mut self) -> Option<geng::Transition> {
        self.transition.take()
    }
}

pub fn run(geng: &Geng, addr: &str) -> impl geng::State {
    let future = {
        let geng = geng.clone();
        let addr = addr.to_owned();
        async move {
            ConfigScreen::new(
                &geng,
                geng::LoadAsset::load(&geng, &run_dir().join("assets"))
                    .await
                    .unwrap(),
                &addr,
            )
        }
    };
    geng::LoadingScreen::new(geng, geng::EmptyLoadingScreen, future, |state| state)
}

fn generate_background(geng: &Geng, assets: &Assets) -> ugli::Texture {
    let mut jigsaw = jigsaw::Jigsaw::generate(geng.ugli(), 0, vec2(40.0, 30.0), vec2(40, 30));
    let camera = geng::Camera2d {
        center: vec2(40.0, 30.0) / 2.0,
        rotation: 0.0,
        fov: 40.0 / 2.0,
    };

    let mesh: Vec<jigsaw::JigsawVertex> = jigsaw
        .tiles
        .iter_mut()
        .flat_map(|tile| {
            tile.interpolated
                .teleport(tile.interpolated.get() * 1.05, Vec2::ZERO);
            let matrix = tile.matrix();
            tile.mesh.iter().map(move |&(mut v)| {
                let pos = matrix * v.a_pos.extend(1.0);
                v.a_pos = pos.xy() / pos.z;
                v
            })
        })
        .collect();
    let mesh = ugli::VertexBuffer::new_dynamic(geng.ugli(), mesh);
    let tiles = jigsaw.tiles;

    let jigsaw_texture =
        ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::new(0.2, 0.2, 0.2, 1.0));
    let mut texture = ugli::Texture::new_with(geng.ugli(), vec2(1600, 900), |_| Rgba::BLACK);
    {
        let framebuffer = &mut ugli::Framebuffer::new_color(
            geng.ugli(),
            ugli::ColorAttachment::Texture(&mut texture),
        );
        ugli::draw(
            framebuffer,
            &assets.shaders.jigsaw,
            ugli::DrawMode::Triangles,
            &mesh,
            (
                ugli::uniforms! {
                    u_model_matrix: Mat3::identity(),
                    u_texture: &jigsaw_texture,
                },
                geng::camera2d_uniforms(&camera, framebuffer.size().map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                // blend_mode: Some(ugli::BlendMode::default()),
                // depth_func: Some(ugli::DepthFunc::Less),
                ..Default::default()
            },
        );
        for tile in tiles {
            let matrix = tile.matrix();
            let outline_color = Rgba::BLACK;
            let depth = 0.0;
            ugli::draw(
                framebuffer,
                &assets.shaders.outline,
                ugli::DrawMode::LineLoop { line_width: 1.0 },
                &tile.outline,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                        u_color: outline_color,
                        u_depth: depth,
                    },
                    geng::camera2d_uniforms(&camera, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..Default::default()
                },
            );
        }
    }

    texture
}
