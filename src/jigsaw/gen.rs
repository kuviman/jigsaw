use super::*;

type Mesh = Vec<[JigsawVertex; 3]>;

pub fn generate_jigsaw(ugli: &Ugli, size: Vec2<f32>, pieces: Vec2<usize>) -> Vec<JigsawMesh> {
    finalize_meshes(
        ugli,
        scale(size, triangulate(pieces, randomize(uniform(pieces)))),
    )
}

fn finalize_meshes(ugli: &Ugli, meshes: Vec<Mesh>) -> Vec<JigsawMesh> {
    meshes
        .into_iter()
        .map(|mesh| ugli::VertexBuffer::new_dynamic(ugli, mesh.into_iter().flatten().collect()))
        .collect()
}

type Polygon = Vec<Vec2<f32>>;

fn uniform(size: Vec2<usize>) -> Vec<Polygon> {
    // TODO: actual jigsaw - not just rectangles
    let tile_size = vec2(1.0, 1.0) / size.map(|x| x as f32);
    (0..size.y)
        .flat_map(|y| {
            (0..size.x).map(move |x| {
                let pos = vec2(x as f32, y as f32) * tile_size;
                vec![
                    pos,
                    pos + vec2(tile_size.x, 0.0),
                    pos + tile_size,
                    pos + vec2(0.0, tile_size.y),
                ]
            })
        })
        .collect()
}

fn randomize(polygons: Vec<Polygon>) -> Vec<Polygon> {
    // TODO
    polygons
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
