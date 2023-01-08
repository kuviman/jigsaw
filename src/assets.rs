use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    pub sounds: Sounds,
    #[asset(range = "1..=1", path = "images/*.png")]
    pub images: Vec<ugli::Texture>,
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub jigsaw: ugli::Program,
    pub outline: ugli::Program,
}

#[derive(geng::Assets)]
pub struct Sounds {
    pub connect_piece: geng::Sound,
    pub grab: geng::Sound,
}
