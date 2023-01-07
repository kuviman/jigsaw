use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    SetupId(Id, String),
    PlayerDisconnected(Id),
    UpdatePos(Id, Vec2<f32>),
    TileGrabbed { player: Id, tile: usize },
    TileReleased { player: Id, tile: usize },
    ConnectTiles(usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SelectRoom(Option<String>),
    UpdatePos(Vec2<f32>),
    GrabTile(usize),
    ReleaseTile(usize),
    ConnectTiles(usize, usize),
}
