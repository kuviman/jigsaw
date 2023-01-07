use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    pub puzzle: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub jigsaw: ugli::Program,
}
