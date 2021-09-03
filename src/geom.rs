use crate::point::Vec2DDrawing;

pub fn bbox_from_points<I>(points: I) -> [[f64; 2]; 2]
    where I: Iterator<Item=Vec2DDrawing>
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

pub fn project(point: Vec2DDrawing, [a, b]: [Vec2DDrawing; 2]) -> Vec2DDrawing {
    // https://en.wikibooks.org/wiki/Linear_Algebra/Orthogonal_Projection_Onto_a_Line

    // If the two points that define the segment are equal they the projection is any of them
    if a == b {
        return a;
    }

    let v = point - a;
    let s = b - a;

    s * (v.dot(s) / s.dot(s)) + a
}

pub fn is_within(point: Vec2DDrawing, a: Vec2DDrawing, b: Vec2DDrawing) -> bool {
    let min = a.min(b);
    let max = a.max(b);

    point.x >= min.x && point.x <= max.x &&
    point.y >= min.y && point.y <= max.y
}

pub fn segment_intersects_circle([a, b]: [Vec2DDrawing; 2], center: Vec2DDrawing, radius: f64) -> bool {
    let proj = project(center, [a, b]);

    proj.distance(center) < radius && is_within(proj, a, b)
}

#[cfg(test)]
mod tests {
    use super::segment_intersects_circle;
    use crate::point::Vec2DDrawing;

    #[test]
    fn test_segment_intersects_circle() {
        assert!(segment_intersects_circle([Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(100.0, 0.0)], Vec2DDrawing::new(50.0, 9.0), 10.0));
    }

    #[test]
    fn test_single_point_segment_intersects_circle() {
        assert!(segment_intersects_circle([Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(0.0, 0.0)], Vec2DDrawing::new(0.0, 0.0), 10.0));
    }
}
