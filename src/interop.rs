use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    SetupId(Id),
    PlayerDisconnected(Id),
    UpdatePos(Id, Vec2<f32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    UpdatePos(Vec2<f32>),
}
