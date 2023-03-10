use super::*;

type Mesh = Vec<[JigsawVertex; 3]>;

pub fn generate_jigsaw(
    ugli: &Ugli,
    seed: u64,
    size: Vec2<f32>,
    pieces: Vec2<usize>,
) -> Vec<(JigsawMesh, ugli::VertexBuffer<JigsawVertex>)> {
    let outlines = outline_vertices(size, pieces, jigsaw(seed, size, pieces));
    let triangles = triangulate(&outlines);
    finalize_meshes(ugli, triangles, outlines)
}

fn finalize_meshes(
    ugli: &Ugli,
    triangles: Vec<Mesh>,
    outlines: Vec<Vec<JigsawVertex>>,
) -> Vec<(JigsawMesh, ugli::VertexBuffer<JigsawVertex>)> {
    triangles
        .into_iter()
        .zip(outlines)
        .map(|(mesh, outline)| {
            (
                ugli::VertexBuffer::new_dynamic(ugli, mesh.into_iter().flatten().collect()),
                ugli::VertexBuffer::new_dynamic(ugli, outline),
            )
        })
        .collect()
}

type Polygon = Vec<Vec2<f32>>;

fn jigsaw(seed: u64, size: Vec2<f32>, pieces: Vec2<usize>) -> Vec<Polygon> {
    let mut rng = rand::prelude::StdRng::seed_from_u64(seed);
    let tile_size = size / pieces.map(|x| x as f32);
    let mut vertices: Vec<Vec2<f32>> = (0..=pieces.y)
        .flat_map(|y| (0..=pieces.x).map(move |x| vec2(x as f32, y as f32) * tile_size))
        .collect();
    // Apply noise
    let dx = tile_size.x * 0.05;
    let dy = tile_size.y * 0.05;
    for (i, v) in vertices.iter_mut().enumerate() {
        if i < pieces.x + 1
            || i >= pieces.y * (pieces.x + 1)
            || i % (pieces.x + 1) == 0
            || i % (pieces.x + 1) == pieces.x
        {
            continue;
        }
        *v += vec2(rng.gen_range(-dx..=dx), rng.gen_range(-dy..=dy));
    }
    let mut jigsaw: Vec<[Vec<Vec2<f32>>; 4]> = (0..pieces.y)
        .flat_map(|y| {
            let vertices = &vertices;
            (0..pieces.x).map(move |x| {
                let i = x + y * (pieces.x + 1);
                [
                    vec![vertices[i]],
                    vec![vertices[i + 1]],
                    vec![vertices[i + 1 + pieces.x + 1]],
                    vec![vertices[i + pieces.x + 1]],
                ]
            })
        })
        .collect();

    let vertical_edges = pieces.y * (pieces.x - 1);
    let edges_count = vertical_edges + pieces.x * (pieces.y - 1);
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
            let start = 0.3 + t * 0.2;
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

        if rng.gen() {
            // Flip edge
            edge.iter_mut().for_each(|v| *v = vec2(v.x, -v.y));
        }

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

fn outline_vertices(
    size: Vec2<f32>,
    pieces: Vec2<usize>,
    polygons: Vec<Polygon>,
) -> Vec<Vec<JigsawVertex>> {
    polygons
        .into_iter()
        .enumerate()
        .map(|(i, polygon)| {
            let center = vec2(i % pieces.x, i / pieces.x).map(|x| x as f32 + 0.5)
                / pieces.map(|x| x as f32)
                * size;
            polygon
                .into_iter()
                .map(|v| JigsawVertex {
                    a_pos: v - center,
                    a_uv: v / size,
                })
                .collect()
        })
        .collect()
}

fn triangulate(polygons: &[Vec<JigsawVertex>]) -> Vec<Mesh> {
    polygons
        .iter()
        .map(|polygon| {
            let flat_polygon: Vec<f32> = polygon
                .iter()
                .flat_map(|v| [v.a_pos.x, v.a_pos.y])
                .collect();
            let triangles =
                earcutr::earcut(&flat_polygon, &[], 2).expect("Failed to triangulate mesh");
            triangles
                .chunks(3)
                .map(|triangle| {
                    let triangle = [triangle[0], triangle[1], triangle[2]];
                    triangle.map(|i| polygon[i])
                })
                .collect()
        })
        .collect()
}
