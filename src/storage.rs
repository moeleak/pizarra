use rstar::Envelope;
use std::collections::HashMap;
use std::f64;

use rstar::{RTree, RTreeObject, AABB, SelectionFunction};

use super::shape::{Shape, ShapeId};
use super::draw_commands::DrawCommand;
use crate::zoom::Zoom;

struct BBoxIdSelection {
    bbox: AABB<[f64; 2]>,
    id: ShapeId,
}

impl BBoxIdSelection {
    fn new(bbox: AABB<[f64; 2]>, id: ShapeId) -> BBoxIdSelection {
        BBoxIdSelection {
            bbox,
            id,
        }
    }
}

impl SelectionFunction<Shape> for BBoxIdSelection {
    fn should_unpack_parent(&self, envelope: &<Shape as RTreeObject>::Envelope) -> bool {
        envelope.contains_envelope(&self.bbox)
    }

    fn should_unpack_leaf(&self, leaf: &Shape) -> bool {
        leaf.id() == self.id
    }
}

fn scale(bbox: AABB<[f64; 2]>, relative: i32) -> AABB<[f64; 2]> {
    let lower = bbox.lower();
    let upper = bbox.upper();

    AABB::from_corners(
        [
            lower[0] * 2.0_f64.powi(relative*-1),
            lower[1] * 2.0_f64.powi(relative*-1),
        ],
        [
            upper[0] * 2.0_f64.powi(relative*-1),
            upper[1] * 2.0_f64.powi(relative*-1),
        ],
    )
}

/// This struct handles storage and fast retrieval of shapes.
pub struct ShapeStorage {
    shape_count: usize,
    shapes: HashMap<ShapeId, (Zoom, AABB<[f64; 2]>)>,
    layers: HashMap<Zoom, RTree<Shape>>,
    current: Option<(Shape, Zoom)>,
}

/// A storage struct that organizes shapes by their zoom level and allows for
/// fast queries given a zoom and a bbox.
impl ShapeStorage {
    pub fn new() -> ShapeStorage {
        ShapeStorage {
            shape_count: 0,
            layers: HashMap::new(),
            shapes: HashMap::new(),
            current: None,
        }
    }

    pub fn add(&mut self, shape: Shape, zoom: Zoom) -> ShapeId {
        // The previous shape has been finished, move it to the immutable storage
        self.flush();

        let id = shape.id();

        self.current = Some((shape, zoom));

        id
    }

    pub fn flush(&mut self) -> Option<ShapeId> {
        let old_current = self.current.take();

        if let Some((old_shape, old_zoom)) = old_current {
            let old_id = old_shape.id();
            let subtree = self.layers.entry(old_zoom).or_insert(RTree::new());

            self.shapes.insert(old_id, (old_zoom, old_shape.envelope()));
            subtree.insert(old_shape);

            self.shape_count += 1;

            Some(old_id)
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ShapeId) -> Option<(Shape, Zoom)> {
        if let Some((zoom, bbox)) = self.shapes.get(&id) {
            let maybe_shape = self.layers.get_mut(zoom).map(|tree| {
                tree.remove_with_selection_function(BBoxIdSelection::new(*bbox, id))
            }).flatten().map(|shape| (shape, *zoom));

            if maybe_shape.is_some() {
                self.shape_count -= 1;
            }

            maybe_shape
        } else {
            None
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut Shape> {
        self.current.as_mut().map(|(shape, _)| shape)
    }

    pub fn shape_count(&self) -> usize {
        self.shape_count
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn draw_commands(&self, zoom: Zoom, bbox: AABB<[f64; 2]>, buffer: &mut Vec<DrawCommand>) {
        unimplemented!()
    }

    pub fn iter_layers(&self) -> impl Iterator<Item=(&Zoom, &RTree<Shape>)> {
        self.layers.iter()
    }

    pub fn get_bounds(&self, zoom: Zoom) -> Option<[[f64; 2]; 2]> {
        self.layers.get(&zoom).map(|tree| {
            tree.iter().map(|shape| {
                shape.envelope()
            }).fold([[f64::MAX, f64::MAX], [f64::MIN, f64::MIN]], |acc, cur| {
                let lower = cur.lower();
                let upper = cur.upper();

                [
                    [
                        if lower[0] < acc[0][0] { lower[0] } else { acc[0][0] },
                        if lower[1] < acc[0][1] { lower[1] } else { acc[0][1] },
                    ],
                    [
                        if upper[0] > acc[1][0] { upper[0] } else { acc[1][0] },
                        if upper[1] > acc[1][1] { upper[1] } else { acc[1][1] },
                    ],
                ]
            })
        })
    }
}

pub struct ShapeIterator<'a> {
    iterator: std::slice::Iter<'a, Shape>,
}

impl <'a> Iterator for ShapeIterator<'a> {
    type Item = &'a Shape;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

#[cfg(test)]
mod tests {
    use rstar::AABB;

    use super::{ShapeStorage, Zoom};

    use crate::shape::{Shape, ShapeType, ShapeId, Line, Rectangle, Ellipse};
    use crate::color::Color;
    use crate::draw_commands::DrawCommand;
    use crate::point::Point;

    #[test]
    fn test_add_shapes_at_zoom() {
        let mut storage = ShapeStorage::new();
        let shapes: Vec<(Shape, Zoom)> = vec![
            (Line::new(Color::Red, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(1)), Zoom::from(0)),
            (Rectangle::new(Color::Green, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(2)), Zoom::from(1)),
            (Ellipse::new(Color::Blue, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(3)), Zoom::from(-1)),
        ];
        let ids: Vec<_> = shapes.into_iter().map(|(shape, zoom)| {
            storage.add(shape, zoom)
        }).collect();

        storage.flush();

        assert_eq!(storage.shape_count(), 3);
        assert_eq!(storage.layer_count(), 3);

        assert_eq!(ids[0], ShapeId::from(1));
        assert_eq!(ids[1], ShapeId::from(2));
        assert_eq!(ids[2], ShapeId::from(3));
    }

    #[test]
    fn test_last_mut() {
        let mut storage = ShapeStorage::new();

        assert!(storage.last_mut().is_none());

        storage.add(Line::new(Color::Blue, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(1)), Zoom::from(0));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Line);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));

        storage.add(Rectangle::new(Color::Blue, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(2)), Zoom::from(0));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Rectangle);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));

        storage.add(Ellipse::new(Color::Blue, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(2)), Zoom::from(0));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Ellipse);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));
    }

    #[test]
    fn test_flush() {
        let mut storage = ShapeStorage::new();

        assert!(storage.last_mut().is_none());

        storage.add(Line::new(Color::Blue, [[0.0, 0.0], [0.0, 0.0]], ShapeId::from(1)), Zoom::from(0));

        assert!(storage.last_mut().is_some());

        storage.flush();

        assert!(storage.last_mut().is_none());
    }

    #[test]
    #[ignore]
    fn test_remove_by_coordinate_radius_and_zoom() {
        assert!(false, "returns enough data to put the shapes again in their places");
        assert!(false, "the data is for restoring the shape in case of a ctrl-z");
        assert!(false, "only visible shapes given the zoom should be deleted.");
        assert!(false, "if there are shapes behind other shapes only delete the one in the front");
    }

    #[test]
    fn test_remove_by_id() {
        let mut storage = ShapeStorage::new();

        assert_eq!(storage.shape_count(), 0);

        let id_to_remove = ShapeId::from(1);
        let id_to_interfere = ShapeId::from(2);

        let obj_to_remove = Line::new(Color::Green, [[0.0, 0.0], [1.0, 1.0]], id_to_remove);
        let obj_to_interfere = Line::new(Color::Blue, [[1.0, 0.0], [0.0, 1.0]], id_to_interfere);

        let id = storage.add(obj_to_remove, Zoom::from(0));
        storage.add(obj_to_interfere, Zoom::from(0));

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.0.id(), id_to_remove);
    }

    #[test]
    fn test_remove_by_id_2() {
        let mut storage = ShapeStorage::new();

        assert_eq!(storage.shape_count(), 0);

        let id_to_remove = ShapeId::from(1);
        let id_to_interfere = ShapeId::from(2);

        let obj_to_remove = Line::new(Color::Green, [[0.0, 0.0], [1.0, 1.0]], id_to_remove);
        let obj_to_interfere = Line::new(Color::Blue, [[1.0, 0.0], [0.0, 1.0]], id_to_interfere);

        storage.add(obj_to_interfere, Zoom::from(0));
        let id = storage.add(obj_to_remove, Zoom::from(0));

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.0.id(), id_to_remove);
    }

    #[test]
    fn test_iter_by_bounds() {
        let mut storage = ShapeStorage::new();

        let shapes = vec![
            // in green the ones that go
            (Rectangle::new(Color::Green, [[-1.1, -1.1], [1.1, 1.1]], ShapeId::from(1)), Zoom::from(-1)), // A
            (Rectangle::new(Color::Green, [[-1.5, 0.5], [-0.5, 1.5]], ShapeId::from(2)), Zoom::from(0)), // B
            (Rectangle::new(Color::Green, [[0.25, -0.5], [0.5, -0.25]], ShapeId::from(3)), Zoom::from(1)), // C

            // in red the ones that dont
            (Rectangle::new(Color::Red, [[20.0, 20.0], [20.0, 20.0]], ShapeId::from(2)), Zoom::from(0)),
        ];

        for (shape, zoom) in shapes.into_iter() {
            storage.add(shape, zoom);
        }

        storage.flush();

        let mut buffer = Vec::new();

        storage.draw_commands(Zoom::from(0), AABB::from_corners([-1.0, -1.0], [1.0, 1.0]), &mut buffer);

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::Green,
                corner_1: Point::new(0.5, -1.0),
                corner_2: Point::new(0.5, 0.5),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::Green,
                corner_1: Point::new(-0.55, -0.55),
                corner_2: Point::new(1.1, 1.1),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::Green,
                corner_1: Point::new(-1.5, 0.5),
                corner_2: Point::new(1.0, 1.0),
            }
        ));

        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_last_shape() {
        let mut storage = ShapeStorage::new();

        // A rectangle of 2x2 in zoom 0 looks like one of 1x1 when looked from zoom 0
        storage.add(Rectangle::new(Color::Red, [[1.0, 1.0], [3.0, 3.0]], ShapeId::from(2)), Zoom::from(-1));

        // only the previous shape is in permanent storage
        storage.flush();

        // A rectangle of 2x2 in zoom 0 aournd the oriign
        storage.add(Rectangle::new(Color::Blue, [[-1.0, -1.0], [1.0, 1.0]], ShapeId::from(1)), Zoom::from(0));

        let mut buffer = Vec::new();

        storage.draw_commands(Zoom::from(0), AABB::from_corners([-2.0, -2.0], [4.0, 4.0]), &mut buffer);

        assert_eq!(buffer, vec![
            DrawCommand::Rectangle{
                color: Color::Red,
                corner_1: Point::new(0.5, 0.5),
                corner_2: Point::new(1.0, 1.0),
            },
            DrawCommand::Rectangle{
                color: Color::Blue,
                corner_1: Point::new(-1.0, -1.0),
                corner_2: Point::new(2.0, 2.0),
            },
        ]);
    }

    #[test]
    fn test_relative_coordinates() {
        let mut storage = ShapeStorage::new();

        // A rectangle of 2x2 in zoom 0 aournd the oriign
        storage.add(Rectangle::new(Color::Blue, [[-1.0, -1.0], [1.0, 1.0]], ShapeId::from(1)), Zoom::from(0));

        // A rectangle of 2x2 in zoom 0 looks like one of 1x1 when looked from zoom 0
        storage.add(Rectangle::new(Color::Red, [[1.0, 1.0], [3.0, 3.0]], ShapeId::from(2)), Zoom::from(-1));

        storage.flush();

        let mut buffer = Vec::new();

        storage.draw_commands(Zoom::from(0), AABB::from_corners([-2.0, -2.0], [4.0, 4.0]), &mut buffer);

        assert_eq!(storage.shape_count(), 2);

        assert!(buffer.contains(&DrawCommand::Rectangle{
            color: Color::Blue,
            corner_1: Point::new(-1.0, -1.0),
            corner_2: Point::new(2.0, 2.0),
        }));

        assert!(buffer.contains(&DrawCommand::Rectangle{
            color: Color::Red,
            corner_1: Point::new(0.5, 0.5),
            corner_2: Point::new(1.0, 1.0),
        }));
    }

    #[test]
    #[ignore]
    fn test_scale() {
        assert!(false, "test proper scaling of AABB objects in the four cuadrants and mixed situations");
    }

    #[test]
    fn test_get_bounds() {
        let mut storage = ShapeStorage::new();

        storage.add(Rectangle::new(Color::Blue, [[-5.0, -5.0], [1.0, 1.0]], ShapeId::from(1)), Zoom::from(0));
        storage.add(Rectangle::new(Color::Blue, [[5.0, 5.0], [1.0, 1.0]], ShapeId::from(1)), Zoom::from(0));

        storage.flush();

        assert_eq!(storage.get_bounds(Zoom::from(0)), Some([[-5.0, -5.0], [5.0, 5.0]]));
    }
}
