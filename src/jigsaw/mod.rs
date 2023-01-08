use super::*;

mod gen;

pub type JigsawMesh = ugli::VertexBuffer<JigsawVertex>;

#[derive(ugli::Vertex, Debug, Clone, Copy)]
pub struct JigsawVertex {
    pub a_pos: Vec2<f32>,
    pub a_uv: Vec2<f32>,
}

pub struct Jigsaw {
    pub tile_size: Vec2<f32>,
    pub tiles: Vec<JigsawTile>,
}

pub struct JigsawTile {
    pub interpolated: Interpolated<Vec2<f32>>,
    pub last_interaction_time: f32,
    pub grabbed_by: Option<Id>,
    pub connected_to: Vec<usize>,
    pub puzzle_pos: Vec2<usize>,
    pub mesh: JigsawMesh,
    pub outline: ugli::VertexBuffer<JigsawVertex>,
}

impl Jigsaw {
    pub fn generate(ugli: &Ugli, seed: u64, size: Vec2<f32>, pieces: Vec2<usize>) -> Self {
        let tile_size = size / pieces.map(|x| x as f32);
        Self {
            tile_size,
            tiles: gen::generate_jigsaw(ugli, seed, size, pieces)
                .into_iter()
                .enumerate()
                .map(|(i, (mesh, outline))| {
                    let puzzle_pos = vec2(i % pieces.x, i / pieces.x);
                    JigsawTile {
                        interpolated: Interpolated::new(
                            puzzle_pos.map(|x| x as f32 + 0.5) * tile_size,
                            Vec2::ZERO,
                        ),
                        last_interaction_time: 0.0,
                        grabbed_by: None,
                        connected_to: vec![],
                        puzzle_pos,
                        mesh,
                        outline,
                    }
                })
                .collect(),
        }
    }

    pub fn get_all_connected(&self, tile: usize) -> HashSet<usize> {
        fn walk_rec(tiles: &[jigsaw::JigsawTile], tile: usize, checked: &mut HashSet<usize>) {
            if !checked.insert(tile) {
                return;
            }
            for &tile in &tiles[tile].connected_to {
                walk_rec(tiles, tile, checked);
            }
        }

        let mut connected = HashSet::new();
        walk_rec(&self.tiles, tile, &mut connected);
        connected
    }
}

impl JigsawTile {
    pub fn matrix(&self) -> Mat3<f32> {
        Mat3::translate(self.interpolated.get())
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
