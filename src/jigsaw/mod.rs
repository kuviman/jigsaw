use super::*;

mod gen;

pub type JigsawMesh = ugli::VertexBuffer<JigsawVertex>;

#[derive(ugli::Vertex, Debug, Clone, Copy)]
pub struct JigsawVertex {
    pub a_pos: Vec2<f32>,
    pub a_uv: Vec2<f32>,
}

pub struct Jigsaw {
    pub tiles: Vec<JigsawTile>,
}

pub struct JigsawTile {
    pub pos: Vec2<f32>,
    pub grabbed_by: Option<Id>,
    pub puzzle_pos: Vec2<usize>,
    pub mesh: JigsawMesh,
}

impl Jigsaw {
    pub fn generate(ugli: &Ugli, size: Vec2<f32>, pieces: Vec2<usize>) -> Self {
        let tile_size = size / pieces.map(|x| x as f32);
        Self {
            tiles: gen::generate_jigsaw(ugli, size, pieces)
                .into_iter()
                .enumerate()
                .map(|(i, mesh)| {
                    let puzzle_pos = vec2(i % pieces.x, i / pieces.x);
                    JigsawTile {
                        pos: puzzle_pos.map(|x| x as f32 + 0.5) * tile_size,
                        grabbed_by: None,
                        puzzle_pos,
                        mesh,
                    }
                })
                .collect(),
        }
    }
}

impl JigsawTile {
    pub fn matrix(&self) -> Mat3<f32> {
        Mat3::translate(self.pos)
    }

    pub fn contains(&self, pos: Vec2<f32>) -> bool {
        let matrix = self.matrix();
        for triangle in self.mesh.chunks(3) {
            let triangle = [triangle[0], triangle[1], triangle[2]].map(|p| {
                let p = matrix * p.a_pos.extend(1.0);
                p.xy() / p.z
            });
            if util::triangle_contains(triangle, pos) {
                return true;
            }
        }
        false
    }
}
