use super::*;

pub fn triangle_contains(tri: [Vec2<f32>; 3], pos: Vec2<f32>) -> bool {
    let d0 = line_signed_d(pos, tri[0], tri[1]);
    let d1 = line_signed_d(pos, tri[1], tri[2]);
    let d2 = line_signed_d(pos, tri[2], tri[0]);

    let has_neg = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let has_pos = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;

    !(has_neg && has_pos)
}

pub fn line_signed_d(p0: Vec2<f32>, p1: Vec2<f32>, p2: Vec2<f32>) -> f32 {
    (p0.x - p2.x) * (p1.y - p2.y) - (p1.x - p2.x) * (p0.y - p2.y)
}
