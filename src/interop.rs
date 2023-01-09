use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    SetupId(Id, RoomConfig),
    RoomNotFound,
    RoomCreated(String),
    PlayerDisconnected(Id),
    UpdatePos(Id, Vec2<f32>),
    UpdatePlayerName(Id, String),
    TileGrabbed {
        player: Id,
        tile: usize,
        offset: Vec2<f32>,
    },
    TileReleased {
        player: Id,
        tile: usize,
        pos: Vec2<f32>,
    },
    ConnectTiles(usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    UpdateName(String),
    CreateRoom(RoomConfig),
    SelectRoom(String),
    UpdatePos(Vec2<f32>),
    GrabTile { tile: usize, offset: Vec2<f32> },
    ReleaseTile(usize, Vec2<f32>),
    ConnectTiles(usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub seed: u64,
    pub size: Vec2<usize>,
    pub image: usize,
}
