use super::*;

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }
}

impl geng::net::server::App for App {
    type Client = Client;
    type ServerMessage = ServerMessage;
    type ClientMessage = ClientMessage;
    fn connect(&mut self, sender: Box<dyn geng::net::Sender<ServerMessage>>) -> Client {
        Client { sender }
    }
}

pub struct Client {
    sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

impl geng::net::Receiver<ClientMessage> for Client {
    fn handle(&mut self, message: ClientMessage) {
        match message {}
    }
}
