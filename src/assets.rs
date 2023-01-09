use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    pub sounds: Sounds,
    #[asset(range = "1..=3", path = "images/*.png")]
    pub images: Vec<ugli::Texture>,
    pub hand: HandAssets,
}

#[derive(geng::Assets)]
pub struct HandAssets {
    pub grab: ugli::Texture,
    pub regular: ugli::Texture,
    pub thumb: ugli::Texture,
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
    #[asset(path = "music.mp3", postprocess = "make_looped")]
    pub music: geng::Sound,
}

fn make_looped(sound: &mut geng::Sound) {
    sound.looped = true;
}
