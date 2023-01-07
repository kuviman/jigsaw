use super::*;

type Mesh = Vec<[JigsawVertex; 3]>;

pub fn generate_jigsaw(ugli: &Ugli, size: Vec2<f32>, pieces: Vec2<usize>) -> Vec<JigsawMesh> {
    finalize_meshes(ugli, scale(size, triangulate(pieces, jigsaw(pieces))))
}

fn finalize_meshes(ugli: &Ugli, meshes: Vec<Mesh>) -> Vec<JigsawMesh> {
    meshes
        .into_iter()
        .map(|mesh| ugli::VertexBuffer::new_dynamic(ugli, mesh.into_iter().flatten().collect()))
        .collect()
}

type Polygon = Vec<Vec2<f32>>;

fn jigsaw(size: Vec2<usize>) -> Vec<Polygon> {
    let tile_size = vec2(1.0, 1.0) / size.map(|x| x as f32);
    let mut jigsaw: Vec<[Vec<Vec2<f32>>; 4]> = (0..size.y)
        .flat_map(|y| {
            (0..size.x).map(move |x| {
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

    let vertical_edges = size.y * (size.x - 1);
    let edges_count = vertical_edges + size.x * (size.y - 1);
    let mut rng = geng::prelude::thread_rng();
    let knob = vec![
        vec2(0.0, 0.0275),
        vec2(-0.058, 0.144),
        vec2(0.0, 0.2725),
        vec2(0.1, 0.3081),
        vec2(0.2, 0.2725),
        vec2(0.258, 0.144),
        vec2(0.2, 0.0275),
    ];
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
            i / size.y + i % size.y * size.x
        } else {
            i - vertical_edges
        };
        let pos = vec2(tile % size.x, tile / size.x).map(|x| x as f32) * tile_size;
        let other = if vertical { tile + 1 } else { tile + size.x };
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

fn triangulate(pieces: Vec2<usize>, polygons: Vec<Polygon>) -> Vec<Mesh> {
    polygons
        .into_iter()
        .enumerate()
        .map(|(i, polygon)| {
            let center =
                vec2(i % pieces.x, i / pieces.x).map(|x| x as f32 + 0.5) / pieces.map(|x| x as f32);

            let flat_polygon: Vec<f32> = polygon.iter().flat_map(|v| [v.x, v.y]).collect();
            let triangles =
                earcutr::earcut(&flat_polygon, &[], 2).expect("Failed to triangulate mesh");
            triangles
                .chunks(3)
                .map(|triangle| {
                    let triangle = [triangle[0], triangle[1], triangle[2]];
                    triangle.map(|i| JigsawVertex {
                        a_pos: polygon[i] - center,
                        a_uv: polygon[i],
                    })
                })
                .collect()
        })
        .collect()
}

fn scale(size: Vec2<f32>, mut polygons: Vec<Mesh>) -> Vec<Mesh> {
    polygons.iter_mut().for_each(|mesh| {
        mesh.iter_mut()
            .for_each(|triangle| triangle.iter_mut().for_each(|v| v.a_pos *= size))
    });
    polygons
}
