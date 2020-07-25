use crate::point::Point;

pub fn bbox_from_points<I>(points: I) -> [[f64; 2]; 2]
    where I: Iterator<Item=Point>
{
    let bbox = points.fold([
        [std::f64::INFINITY, std::f64::INFINITY], // bottom-left corner
        [std::f64::NEG_INFINITY, std::f64::NEG_INFINITY], // top right corner
    ], |acc, point| {
        [
            [
                if point.x < acc[0][0] { point.x } else { acc[0][0] },
                if point.y < acc[0][1] { point.y } else { acc[0][1] },
            ],
            [
                if point.x > acc[1][0] { point.x } else { acc[1][0] },
                if point.y > acc[1][1] { point.y } else { acc[1][1] },
            ],
        ]
    });

    assert_ne!(bbox[0][0], std::f64::INFINITY);
    assert_ne!(bbox[0][1], std::f64::INFINITY);
    assert_ne!(bbox[1][0], std::f64::NEG_INFINITY);
    assert_ne!(bbox[1][1], std::f64::NEG_INFINITY);

    bbox
}

pub fn project(point: Point, [a, b]: [Point; 2]) -> Point {
    // https://en.wikibooks.org/wiki/Linear_Algebra/Orthogonal_Projection_Onto_a_Line
    let v = point - a;
    let s = b - a;

    s * (v.dot(s) / s.dot(s)) + a
}

pub fn is_within(point: Point, a: Point, b: Point) -> bool {
    let min = a.min(b);
    let max = a.max(b);

    point.x >= min.x && point.x <= max.x &&
    point.y >= min.y && point.y <= max.y
}

pub fn segment_intersects_circle([a, b]: [Point; 2], center: Point, radius: f64) -> bool {
    let proj = project(center, [a, b]);

    proj.distance(center) < radius && is_within(proj, a, b)
}
