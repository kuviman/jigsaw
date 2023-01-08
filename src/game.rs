use geng::Camera2d;

use crate::jigsaw::Jigsaw;

use super::*;

const SNAP_DISTANCE: f32 = 0.2;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(HasId)]
struct Player {
    id: Id,
    interpolation: Interpolated<Vec2<f32>>,
    tile_grabbed: Option<(usize, Vec2<f32>)>,
}

struct Game {
    geng: Geng,
    room_config: RoomConfig,
    assets: Rc<Assets>,
    id: Id,
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
        room_config: RoomConfig,
        connection: Connection,
    ) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            id,
            connection,
            players: Collection::new(),
            camera: Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            },
            framebuffer_size: vec2(1, 1),
            jigsaw: {
                let image = &assets.images[room_config.image];
                let size = image.size().map(|x| x as f32);
                let size = size * 5.0 / size.y;
                let seed = thread_rng().gen(); // TODO: get from the room
                let mut jigsaw = Jigsaw::generate(geng.ugli(), seed, size, room_config.size);
                for tile in &mut jigsaw.tiles {
                    tile.pos -= size / 2.0;
                }
                jigsaw
            },
            room_config,
        }
    }
    fn get_player(&mut self, id: Id) -> &mut Player {
        if self.players.get(&id).is_none() {
            self.players.insert(Player {
                id,
                interpolation: Interpolated::new(Vec2::ZERO, Vec2::ZERO),
                tile_grabbed: None,
            });
        }
        self.players.get_mut(&id).unwrap()
    }
    fn handle_connection(&mut self) {
        while let Some(message) = self.connection.try_recv() {
            match message {
                ServerMessage::SetupId(..) => unreachable!(),
                ServerMessage::RoomNotFound => unreachable!(),
                ServerMessage::RoomCreated(..) => unreachable!(),
                ServerMessage::UpdatePos(id, pos) => {
                    self.get_player(id)
                        .interpolation
                        .server_update(pos, Vec2::ZERO);
                }
                ServerMessage::PlayerDisconnected(id) => {
                    self.players.remove(&id);
                }
                ServerMessage::TileGrabbed {
                    player,
                    tile,
                    offset,
                } => {
                    self.players.get_mut(&player).unwrap().tile_grabbed = Some((tile, offset));
                    self.jigsaw.tiles[tile].grabbed_by = Some(player);
                }
                ServerMessage::TileReleased { player, tile, pos } => {
                    let offset = self
                        .players
                        .get_mut(&player)
                        .unwrap()
                        .tile_grabbed
                        .take()
                        .map_or(Vec2::ZERO, |(_, offset)| offset);
                    self.jigsaw.tiles[tile].grabbed_by = None;
                    self.move_tile(tile, pos + offset);
                }
                ServerMessage::ConnectTiles(a, b) => {
                    self.jigsaw.tiles[a].connected_to.push(b);
                    self.jigsaw.tiles[b].connected_to.push(a);
                    let delta = self.jigsaw.tiles[a].puzzle_pos.map(|x| x as i32)
                        - self.jigsaw.tiles[b].puzzle_pos.map(|x| x as i32);
                    let pos = if delta.x == 0 && delta.y.abs() == 1 {
                        // Tile is adjacent vertically
                        self.jigsaw.tiles[b].pos
                            + vec2(0.0, self.jigsaw.tile_size.y * delta.y.signum() as f32)
                    } else if delta.y == 0 && delta.x.abs() == 1 {
                        // Tile is adjacent horizontally
                        self.jigsaw.tiles[b].pos
                            + vec2(self.jigsaw.tile_size.x * delta.x.signum() as f32, 0.0)
                    } else {
                        unreachable!()
                    };
                    self.move_tile(a, pos);
                }
            }
        }
    }
    fn click(&mut self, pos: Vec2<f32>) {
        for (i, tile) in self.jigsaw.tiles.iter_mut().enumerate() {
            if tile.contains(pos) {
                let player = self.players.get_mut(&self.id).unwrap();
                let offset = tile.pos - pos;
                player.tile_grabbed = Some((i, offset));
                tile.grabbed_by = Some(self.id);
                self.connection
                    .send(ClientMessage::GrabTile { tile: i, offset });
            }
        }
    }
    fn release(&mut self) {
        let player = self.players.get_mut(&self.id).unwrap();
        if let Some((tile_id, _)) = player.tile_grabbed.take() {
            self.connection.send(ClientMessage::ReleaseTile(
                tile_id,
                player.interpolation.get(),
            ));
            let tile = self.jigsaw.tiles.get_mut(tile_id).unwrap();
            tile.grabbed_by = None;

            // Try to connect
            let connected = self.jigsaw.get_all_connected(tile_id);
            for tile_id in connected {
                let tile = self.jigsaw.tiles.get(tile_id).unwrap();
                let pos = tile.pos;
                let puzzle_pos = tile.puzzle_pos;
                for (i, other) in self.jigsaw.tiles.iter().enumerate() {
                    if tile.connected_to.contains(&i) {
                        continue;
                    }
                    let delta = puzzle_pos.map(|x| x as i32) - other.puzzle_pos.map(|x| x as i32);
                    let delta = if delta.x == 0 && delta.y.abs() == 1 {
                        // Tile is adjacent vertically
                        Some(
                            pos - other.pos
                                - vec2(0.0, self.jigsaw.tile_size.y * delta.y.signum() as f32),
                        )
                    } else if delta.y == 0 && delta.x.abs() == 1 {
                        // Tile is adjacent horizontally
                        Some(
                            pos - other.pos
                                - vec2(self.jigsaw.tile_size.x * delta.x.signum() as f32, 0.0),
                        )
                    } else {
                        None
                    };
                    if let Some(delta) = delta {
                        // Delta to the snap position
                        if delta.len() <= SNAP_DISTANCE {
                            self.connection
                                .send(ClientMessage::ConnectTiles(tile_id, i));
                        }
                    }
                }
            }
        }
    }
    fn move_tile(&mut self, tile: usize, pos: Vec2<f32>) {
        let tiles = self.jigsaw.get_all_connected(tile);
        let start_pos = self.jigsaw.tiles[tile].puzzle_pos.map(|x| x as i32);
        for tile in tiles {
            let delta = self.jigsaw.tiles[tile].puzzle_pos.map(|x| x as i32) - start_pos;
            self.jigsaw.tiles[tile].pos = pos + delta.map(|x| x as f32) * self.jigsaw.tile_size;
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.handle_connection();
        let mut moves = Vec::new();
        for player in &mut self.players {
            player.interpolation.update(delta_time);

            // Update grabbed tile
            if let Some((tile_id, offset)) = player.tile_grabbed {
                if let Some(tile) = self.jigsaw.tiles.get_mut(tile_id) {
                    if tile.grabbed_by != Some(player.id) {
                        player.tile_grabbed = None;
                    } else {
                        moves.push((tile_id, player.interpolation.get() + offset));
                    }
                }
            }
        }
        for (tile, pos) in moves {
            self.move_tile(tile, pos);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Rgba::new(0.1, 0.1, 0.1, 1.0)), None, None);

        for tile in &self.jigsaw.tiles {
            let matrix = tile.matrix();
            ugli::draw(
                framebuffer,
                &self.assets.shaders.jigsaw,
                ugli::DrawMode::Triangles,
                &tile.mesh,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                        u_texture: &self.assets.images[self.room_config.image],
                    },
                    geng::camera2d_uniforms(&self.camera, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters::default(),
            );
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
        match event {
            geng::Event::MouseMove { position, .. } => {
                let pos = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                self.connection.send(ClientMessage::UpdatePos(pos));
                let me = self.get_player(self.id);
                me.interpolation.server_update(pos, Vec2::ZERO);
                me.interpolation.update(1e5); // HAHA
            }
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                let pos = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                self.click(pos);
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                self.release();
            }
            _ => (),
        }
    }
}

pub fn run(geng: &Geng, addr: &str, room: &str) -> impl geng::State {
    let future = {
        let geng = geng.clone();
        let room = room.to_owned();
        let connection = geng::net::client::connect(addr);
        async move {
            let assets: Rc<Assets> = geng::LoadAsset::load(&geng, &run_dir().join("assets"))
                .await
                .expect("Failed to load assets");
            let mut connection: game::Connection = connection.await;
            connection.send(ClientMessage::SelectRoom(room));
            match connection.next().await {
                Some(ServerMessage::SetupId(id, room_config)) => {
                    game::Game::new(&geng, &assets, id, room_config, connection)
                }
                Some(ServerMessage::RoomNotFound) => panic!("Room not found"),
                _ => unreachable!(),
            }
        }
    };
    geng::LoadingScreen::new(&geng, geng::EmptyLoadingScreen, future, |state| state)
}
