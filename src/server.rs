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

struct JigsawTile {
    grabbed_by: Option<Id>,
}

#[derive(HasId)]
struct Room {
    #[has_id(id)]
    name: String,
    tiles: Vec<JigsawTile>,
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
                    self.rooms.insert(Room {
                        name: name.clone(),
                        tiles: default(),
                        config,
                    });
                    player.sender.send(ServerMessage::RoomCreated(name));
                    break;
                }
            },
            ClientMessage::UpdatePos(pos) => {
                for player in &mut self.players {
                    if player.id != id && player.room == room {
                        player.sender.send(ServerMessage::UpdatePos(id, pos));
                    }
                }
            }
            ClientMessage::SelectRoom(room) => {
                let player = self.players.get_mut(&id).unwrap();
                if let Some(room) = self.rooms.get(&room) {
                    player.room = room.name.clone();
                    player
                        .sender
                        .send(ServerMessage::SetupId(id, room.config.clone()));
                } else {
                    player.sender.send(ServerMessage::RoomNotFound);
                }
            }
            ClientMessage::GrabTile(tile_id) => {
                if let Some(room) = self.rooms.get_mut(&room) {
                    if let Some(tile) = room.tiles.get_mut(tile_id) {
                        if tile.grabbed_by.is_none() {
                            tile.grabbed_by = Some(id);
                            for player in &mut self.players {
                                if player.room == room.name {
                                    player.sender.send(ServerMessage::TileGrabbed {
                                        player: id,
                                        tile: tile_id,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            ClientMessage::ReleaseTile(tile_id) => {
                if let Some(room) = self.rooms.get_mut(&room) {
                    if let Some(tile) = room.tiles.get_mut(tile_id) {
                        if tile.grabbed_by == Some(id) {
                            for player in &mut self.players {
                                if player.room == room.name {
                                    player.sender.send(ServerMessage::TileReleased {
                                        player: id,
                                        tile: tile_id,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            ClientMessage::ConnectTiles(a, b) => {
                // TODO: check validity
                for player in &mut self.players {
                    if player.room == room {
                        player.sender.send(ServerMessage::ConnectTiles(a, b));
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
        let mut player = Player {
            id,
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
