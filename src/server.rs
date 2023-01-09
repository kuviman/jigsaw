use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct IdGen {
    next_id: u64,
}

impl IdGen {
    fn new() -> Self {
        Self { next_id: 0 }
    }
    fn gen(&mut self) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        id
    }
}

#[derive(HasId)]
struct Player {
    id: Id,
    room: String,
    name: String,
    sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

fn create_room() -> String {
    rand::distributions::DistString::sample_string(
        &rand::distributions::Alphanumeric,
        &mut thread_rng(),
        16,
    )
}

struct State {
    id_gen: IdGen,
    players: Collection<Player>,
    rooms: Collection<Room>,
}

#[derive(HasId)]
struct Room {
    #[has_id(id)]
    name: String,
    tiles: Vec<TileState>,
    config: RoomConfig,
}

impl State {
    fn new() -> Self {
        Self {
            id_gen: IdGen::new(),
            players: Collection::new(),
            rooms: Collection::new(),
        }
    }
    fn handle(&mut self, id: Id, message: ClientMessage) {
        let room = self.players.get(&id).unwrap().room.clone();
        match message {
            ClientMessage::CreateRoom(config) => loop {
                let name = create_room();
                if self.rooms.get(&name).is_some() {
                    warn!("Rng room name collision");
                    continue;
                } else {
                    let player = self.players.get_mut(&id).unwrap();
                    let mut rng = thread_rng();
                    let bounds = AABB::ZERO.extend_uniform(3.0);
                    let spawn_area = AABB::point(bounds.bottom_left())
                        .extend_positive(vec2(bounds.width(), 3.0));
                    let tiles = (0..config.size.x * config.size.y)
                        .map(|_| {
                            let pos = vec2(
                                rng.gen_range(spawn_area.x_min..=spawn_area.x_max),
                                rng.gen_range(spawn_area.y_min..=spawn_area.y_max),
                            );
                            TileState {
                                grabbed_by: None,
                                pos,
                                connections: Vec::new(),
                            }
                        })
                        .collect();
                    self.rooms.insert(Room {
                        name: name.clone(),
                        tiles,
                        config,
                    });
                    player.sender.send(ServerMessage::RoomCreated(name));
                    break;
                }
            },
            ClientMessage::UpdatePos(pos) => {
                if let Some(room) = self.rooms.get_mut(&room) {
                    for player in &mut self.players {
                        if player.id != id && player.room == room.name {
                            player.sender.send(ServerMessage::UpdatePos(id, pos));
                        }
                    }
                }
            }
            ClientMessage::UpdateName(name) => {
                self.players.get_mut(&id).unwrap().name = name.clone();
                for player in &mut self.players {
                    if player.id != id && player.room == room {
                        player
                            .sender
                            .send(ServerMessage::UpdatePlayerName(id, name.clone()));
                    }
                }
            }
            ClientMessage::SelectRoom(room) => {
                let player = self.players.get_mut(&id).unwrap();
                let mut messages = Vec::new();
                if let Some(room) = self.rooms.get(&room) {
                    player.room = room.name.clone();
                    player.sender.send(ServerMessage::SetupId {
                        player_id: id,
                        room_config: room.config.clone(),
                        tiles: room.tiles.clone(),
                    });
                    for player in &self.players {
                        if player.id != id && player.room == room.name {
                            messages.push(ServerMessage::UpdatePlayerName(
                                player.id,
                                player.name.clone(),
                            ));
                        }
                    }
                } else {
                    player.sender.send(ServerMessage::RoomNotFound);
                }
                let player = self.players.get_mut(&id).unwrap(); // KEKW
                for message in messages {
                    player.sender.send(message);
                }
            }
            ClientMessage::GrabTile {
                tile: tile_id,
                offset,
            } => {
                if let Some(room) = self.rooms.get_mut(&room) {
                    if let Some(tile) = room.tiles.get_mut(tile_id) {
                        if tile.grabbed_by.is_none() {
                            tile.grabbed_by = Some(id);
                            for player in &mut self.players {
                                if player.id != id && player.room == room.name {
                                    player.sender.send(ServerMessage::TileGrabbed {
                                        player: id,
                                        tile: tile_id,
                                        offset,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            ClientMessage::ReleaseTile(updates) => {
                if let Some(room) = self.rooms.get_mut(&room) {
                    if let Some((tile_id, pos)) = updates.first().copied() {
                        for player in &mut self.players {
                            if player.id != id && player.room == room.name {
                                player.sender.send(ServerMessage::TileReleased {
                                    player: id,
                                    tile: tile_id,
                                    pos,
                                });
                            }
                        }
                    }
                    for (tile_id, pos) in updates {
                        if let Some(tile) = room.tiles.get_mut(tile_id) {
                            tile.grabbed_by.take();
                            tile.pos = pos;
                        }
                    }
                }
            }
            ClientMessage::ConnectTiles(a, b) => {
                // TODO: check validity
                if let Some(room) = self.rooms.get_mut(&room) {
                    room.tiles[a].connections.push(b);
                    room.tiles[b].connections.push(a);
                    for player in &mut self.players {
                        if player.room == room.name {
                            player.sender.send(ServerMessage::ConnectTiles(a, b));
                        }
                    }
                }
            }
        }
    }
}

pub struct App {
    state: Arc<Mutex<State>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::new())),
        }
    }
}

impl geng::net::server::App for App {
    type Client = Client;
    type ServerMessage = ServerMessage;
    type ClientMessage = ClientMessage;
    fn connect(&mut self, sender: Box<dyn geng::net::Sender<ServerMessage>>) -> Client {
        let mut state = self.state.lock().unwrap();
        let id = state.id_gen.gen();
        let player = Player {
            id,
            name: "".to_owned(),
            room: create_room(),
            sender,
        };
        state.players.insert(player);
        Client {
            id,
            state: self.state.clone(),
        }
    }
}

pub struct Client {
    id: Id,
    state: Arc<Mutex<State>>,
}

impl geng::net::Receiver<ClientMessage> for Client {
    fn handle(&mut self, message: ClientMessage) {
        self.state.lock().unwrap().handle(self.id, message);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.players.remove(&self.id);
        for player in &mut state.players {
            player
                .sender
                .send(ServerMessage::PlayerDisconnected(player.id));
        }
    }
}
