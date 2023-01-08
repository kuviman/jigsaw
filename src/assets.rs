use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(range = "1..=1", path = "images/*.png")]
    pub images: Vec<ugli::Texture>,
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub jigsaw: ugli::Program,
}
