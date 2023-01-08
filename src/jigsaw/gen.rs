use super::*;

type Mesh = Vec<[JigsawVertex; 3]>;

pub fn generate_jigsaw(
    ugli: &Ugli,
    seed: u64,
    size: Vec2<f32>,
    pieces: Vec2<usize>,
) -> Vec<JigsawMesh> {
    finalize_meshes(ugli, triangulate(size, pieces, jigsaw(seed, size, pieces)))
}

fn finalize_meshes(ugli: &Ugli, meshes: Vec<Mesh>) -> Vec<JigsawMesh> {
    meshes
        .into_iter()
        .map(|mesh| ugli::VertexBuffer::new_dynamic(ugli, mesh.into_iter().flatten().collect()))
        .collect()
}

type Polygon = Vec<Vec2<f32>>;

fn jigsaw(seed: u64, size: Vec2<f32>, pieces: Vec2<usize>) -> Vec<Polygon> {
    let tile_size = size / pieces.map(|x| x as f32);
    let mut jigsaw: Vec<[Vec<Vec2<f32>>; 4]> = (0..pieces.y)
        .flat_map(|y| {
            (0..pieces.x).map(move |x| {
                let pos = vec2(x as f32, y as f32) * tile_size;
                [
                    vec![pos],
                    vec![pos + vec2(tile_size.x, 0.0)],
                    vec![pos + tile_size],
                    vec![pos + vec2(0.0, tile_size.y)],
                ]
            })
        })
        .collect();

    let vertical_edges = pieces.y * (pieces.x - 1);
    let edges_count = vertical_edges + pieces.x * (pieces.y - 1);
    let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
    let knob: Vec<Vec2<f32>> = {
        const KNOB_RESOLUTION: usize = 10;
        const MIN: f32 = 3.95;
        const MAX: f32 = -0.85;
        (0..KNOB_RESOLUTION)
            .map(|i| {
                let t = i as f32 / (KNOB_RESOLUTION as f32 - 1.0) * (MAX - MIN) + MIN;
                let (sin, cos) = t.sin_cos();
                // (x - 0.1)^2 + (y - 0.15)^2 = 0.025
                vec2(0.1 + cos * 0.15, 0.15 + sin * 0.15)
            })
            .collect()
    };
    let mut edges: Vec<Vec<Vec2<f32>>> = (0..edges_count)
        .map(|i| {
            let t = i as f32 / (edges_count as f32 - 1.0);
            let start = 0.2 + t * 0.4;
            let end = start + 0.2;
            itertools::chain![
                [vec2(start, 0.0)],
                knob.iter().map(|v| *v + vec2(start, 0.0)),
                [vec2(end, 0.0)],
            ]
            .collect()
        })
        .collect();
    edges.shuffle(&mut rng);

    for (i, mut edge) in edges.into_iter().enumerate() {
        let vertical = i < vertical_edges;
        let tile = if vertical {
            i / pieces.y + i % pieces.y * pieces.x
        } else {
            i - vertical_edges
        };
        let pos = vec2(tile % pieces.x, tile / pieces.x).map(|x| x as f32) * tile_size;
        let other = if vertical { tile + 1 } else { tile + pieces.x };
        let scale = if vertical { tile_size.y } else { tile_size.x };
        if vertical {
            edge.iter_mut()
                .for_each(|v| *v = v.rotate_90() * scale + pos + vec2(tile_size.x, 0.0));
        } else {
            edge.iter_mut()
                .for_each(|v| *v = *v * scale + pos + vec2(0.0, tile_size.y));
        }
        if vertical {
            jigsaw[other][3].extend(edge.iter().rev().copied());
            jigsaw[tile][1].extend(edge);
        } else {
            jigsaw[tile][2].extend(edge.iter().rev().copied());
            jigsaw[other][0].extend(edge);
        }
    }

    jigsaw
        .into_iter()
        .map(|sides| sides.into_iter().flatten().collect())
        .collect()
}

fn triangulate(size: Vec2<f32>, pieces: Vec2<usize>, polygons: Vec<Polygon>) -> Vec<Mesh> {
    polygons
        .into_iter()
        .enumerate()
        .map(|(i, polygon)| {
            let center = vec2(i % pieces.x, i / pieces.x).map(|x| x as f32 + 0.5)
                / pieces.map(|x| x as f32)
                * size;

            let flat_polygon: Vec<f32> = polygon.iter().flat_map(|v| [v.x, v.y]).collect();
            let triangles =
                earcutr::earcut(&flat_polygon, &[], 2).expect("Failed to triangulate mesh");
            triangles
                .chunks(3)
                .map(|triangle| {
                    let triangle = [triangle[0], triangle[1], triangle[2]];
                    triangle.map(|i| JigsawVertex {
                        a_pos: polygon[i] - center,
                        a_uv: polygon[i] / size,
                    })
                })
                .collect()
        })
        .collect()
}
