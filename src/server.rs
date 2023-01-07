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
}

impl State {
    fn new() -> Self {
        Self {
            id_gen: IdGen::new(),
            players: Collection::new(),
        }
    }
    fn handle(&mut self, id: Id, message: ClientMessage) {
        let room = self.players.get(&id).unwrap().room.clone();
        match message {
            ClientMessage::UpdatePos(pos) => {
                for player in &mut self.players {
                    if player.id != id && player.room == room {
                        player.sender.send(ServerMessage::UpdatePos(id, pos));
                    }
                }
            }
            ClientMessage::SelectRoom(room) => {
                let player = self.players.get_mut(&id).unwrap();
                player.room = room.unwrap_or_else(create_room);
                player
                    .sender
                    .send(ServerMessage::SetupId(id, player.room.clone()));
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
