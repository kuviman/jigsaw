use geng::prelude::*;

mod assets;
mod game;
mod interop;
mod interpolation;
mod jigsaw;
mod main_menu;
#[cfg(not(target_arch = "wasm32"))]
mod server;
mod slider;
mod splitscreen;
mod util;

use assets::Assets;
use interop::*;
use interpolation::*;
use slider::*;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    pub server: Option<String>,
    #[clap(long)]
    pub connect: Option<String>,
    #[clap(long)]
    pub room: Option<String>,
    #[clap(long)]
    pub splits: Option<usize>,
    #[clap(long)]
    pub room_config: Option<std::path::PathBuf>,
}

fn main() {
    let _ = logger::init();
    geng::setup_panic_handler();
    let mut opt: Opt = program_args::parse();

    if opt.connect.is_none() && opt.server.is_none() {
        if cfg!(target_arch = "wasm32") {
            opt.connect = Some(
                option_env!("CONNECT")
                    .expect("Set CONNECT compile time env var")
                    .to_owned(),
            );
        } else {
            opt.server = Some("127.0.0.1:1155".to_owned());
            opt.connect = Some("ws://127.0.0.1:1155".to_owned());
        }
    }

    if opt.server.is_some() && opt.connect.is_none() {
        #[cfg(not(target_arch = "wasm32"))]
        geng::net::Server::new(server::App::new(), opt.server.as_deref().unwrap()).run();
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        let server = if let Some(addr) = &opt.server {
            let server = geng::net::Server::new(server::App::new(), addr);
            let server_handle = server.handle();
            let server_thread = std::thread::spawn(move || {
                server.run();
            });
            Some((server_handle, server_thread))
        } else {
            None
        };

        let geng = Geng::new_with(geng::ContextOptions {
            title: "LD 52".to_owned(),
            target_ui_resolution: Some(vec2(800.0, 600.0)),
            ..default()
        });
        if let Some(config) = &opt.room_config {
            let config: RoomConfig =
                serde_json::from_reader(std::fs::File::open(config).unwrap()).unwrap();
            futures::executor::block_on(async {
                let mut con: Connection =
                    geng::net::client::connect(opt.connect.as_deref().unwrap()).await;
                con.send(ClientMessage::CreateRoom(config));
                match con.next().await {
                    Some(ServerMessage::RoomCreated(name)) => {
                        opt.room = Some(name);
                    }
                    _ => unreachable!(),
                }
            });
        }
        if let Some(room) = &opt.room {
            geng::run(
                &geng,
                splitscreen::SplitScreen::new(
                    &geng,
                    (0..opt.splits.unwrap_or(1)).map(|_| {
                        Box::new(game::run(&geng, opt.connect.as_deref().unwrap(), room))
                            as Box<dyn geng::State>
                    }),
                ),
            );
        } else {
            geng::run(
                &geng,
                main_menu::run(&geng, opt.connect.as_deref().unwrap()),
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some((server_handle, server_thread)) = server {
            server_handle.shutdown();
            server_thread.join().unwrap();
        }
    }
}
