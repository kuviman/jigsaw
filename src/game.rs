use geng::Camera2d;

use crate::jigsaw::Jigsaw;

use super::*;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(HasId)]
struct Player {
    id: Id,
    interpolation: Interpolated<Vec2<f32>>,
}

struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    id: Id,
    room: String,
    connection: Connection,
    players: Collection<Player>,
    camera: Camera2d,
    framebuffer_size: Vec2<usize>,
    jigsaw: Jigsaw,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        id: Id,
        room: String,
        connection: Connection,
    ) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            id,
            room,
            connection,
            players: Collection::new(),
            camera: Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 30.0,
            },
            framebuffer_size: vec2(1, 1),
            jigsaw: {
                let size = assets.puzzle.size().map(|x| x as f32);
                let size = size * 10.0 / size.y;
                Jigsaw::generate(geng.ugli(), size, size.map(|x| x.floor() as usize))
            },
        }
    }
    fn get_player(&mut self, id: Id) -> &mut Player {
        if self.players.get(&id).is_none() {
            self.players.insert(Player {
                id,
                interpolation: Interpolated::new(Vec2::ZERO, Vec2::ZERO),
            });
        }
        self.players.get_mut(&id).unwrap()
    }
    fn handle_connection(&mut self) {
        while let Some(message) = self.connection.try_recv() {
            match message {
                ServerMessage::SetupId(..) => unreachable!(),
                ServerMessage::UpdatePos(id, pos) => {
                    self.get_player(id)
                        .interpolation
                        .server_update(pos, Vec2::ZERO);
                }
                ServerMessage::PlayerDisconnected(id) => {
                    self.players.remove(&id);
                }
            }
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.handle_connection();
        for player in &mut self.players {
            player.interpolation.update(delta_time);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        for piece in &self.jigsaw.pieces {
            let matrix = Mat3::translate(piece.pos);
            ugli::draw(
                framebuffer,
                &self.assets.shaders.jigsaw,
                ugli::DrawMode::Triangles,
                &piece.mesh,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                        u_texture: &self.assets.puzzle,
                    },
                    geng::camera2d_uniforms(&self.camera, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters::default(),
            )
        }

        for player in &self.players {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(
                    player.interpolation.get(),
                    self.camera.fov * 0.01,
                    Rgba::WHITE,
                ),
            );
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::MouseMove { position, .. } = event {
            let pos = self.camera.screen_to_world(
                self.framebuffer_size.map(|x| x as f32),
                position.map(|x| x as f32),
            );
            self.connection.send(ClientMessage::UpdatePos(pos));
            let me = self.get_player(self.id);
            me.interpolation.server_update(pos, Vec2::ZERO);
            me.interpolation.update(1e5); // HAHA
        }
    }
}

pub fn run(addr: &str, room: Option<String>) {
    let geng = Geng::new_with(geng::ContextOptions {
        title: "LD 52".to_owned(),
        ..default()
    });
    let future = {
        let geng = geng.clone();
        let connection = geng::net::client::connect(addr);
        async move {
            let assets: Rc<Assets> = geng::LoadAsset::load(&geng, &run_dir().join("assets"))
                .await
                .expect("Failed to load assets");
            let mut connection: game::Connection = connection.await;
            connection.send(ClientMessage::SelectRoom(room));
            let Some(ServerMessage::SetupId(id, room)) = connection.next().await else {
                panic!()
            };
            game::Game::new(&geng, &assets, id, room, connection)
        }
    };
    geng::run(
        &geng,
        geng::LoadingScreen::new(&geng, geng::EmptyLoadingScreen, future, |state| state),
    );
}
