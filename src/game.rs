use super::*;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

struct Game {
    connection: Connection,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, connection: Connection) -> Self {
        Self { connection }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

pub fn run(addr: &str) {
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
            let connection: game::Connection = connection.await;
            game::Game::new(&geng, &assets, connection)
        }
    };
    geng::run(
        &geng,
        geng::LoadingScreen::new(&geng, geng::EmptyLoadingScreen, future, |state| state),
    );
}
