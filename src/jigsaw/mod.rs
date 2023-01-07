use super::*;

mod gen;

pub type JigsawMesh = ugli::VertexBuffer<JigsawVertex>;

#[derive(ugli::Vertex, Debug, Clone, Copy)]
pub struct JigsawVertex {
    pub a_pos: Vec2<f32>,
    pub a_uv: Vec2<f32>,
}

pub struct Jigsaw {
    pub pieces: Vec<JigsawPiece>,
}

pub struct JigsawPiece {
    pub pos: Vec2<f32>,
    pub puzzle_pos: Vec2<usize>,
    pub mesh: JigsawMesh,
}

impl Jigsaw {
    pub fn generate(ugli: &Ugli, size: Vec2<f32>, pieces: Vec2<usize>) -> Self {
        let tile_size = size / pieces.map(|x| x as f32);
        Self {
            pieces: gen::generate_jigsaw(ugli, size, pieces)
                .into_iter()
                .enumerate()
                .map(|(i, mesh)| {
                    let puzzle_pos = vec2(i % pieces.x, i / pieces.x);
                    JigsawPiece {
                        pos: puzzle_pos.map(|x| x as f32) * tile_size,
                        puzzle_pos,
                        mesh,
                    }
                })
                .collect(),
        }
    }
}
