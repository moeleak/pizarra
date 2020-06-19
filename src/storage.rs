use rstar::Envelope;
use std::collections::HashMap;
use std::f64;

use rstar::{RTree, RTreeObject, AABB, SelectionFunction};

use super::shape::{Shape, ShapeId};
use super::draw_commands::DrawCommand;
use crate::point::Point;

impl RTreeObject for Shape {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let [p1, p2] = self.bbox();

        AABB::from_corners(p1, p2)
    }
}

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

/// This struct handles storage and fast retrieval of shapes.
pub struct Storage {
    shape_count: usize,
    ids: HashMap<ShapeId, AABB<[f64; 2]>>,
    shapes: RTree<Shape>,
    current: Option<Shape>,
}

/// A storage struct that organizes shapes by their zoom level and allows for
/// fast queries given a zoom and a bbox.
impl Storage {
    pub fn new() -> Storage {
        Storage {
            shape_count: 0,
            shapes: RTree::new(),
            ids: HashMap::new(),
            current: None,
        }
    }

    pub fn add(&mut self, shape: Shape) -> ShapeId {
        // The previous shape has been finished, move it to the immutable storage
        self.flush();

        let id = shape.id();

        self.current = Some(shape);

        id
    }

    pub fn flush(&mut self) -> Option<ShapeId> {
        let old_current = self.current.take();

        if let Some(old_shape) = old_current {
            let old_id = old_shape.id();

            self.ids.insert(old_id, old_shape.envelope());
            self.shapes.insert(old_shape);

            self.shape_count += 1;

            Some(old_id)
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ShapeId) -> Option<Shape> {
        if let Some(bbox) = self.ids.get(&id) {
            let maybe_shape = self.shapes.remove_with_selection_function(BBoxIdSelection::new(*bbox, id));

            if maybe_shape.is_some() {
                self.shape_count -= 1;
            }

            maybe_shape
        } else {
            None
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut Shape> {
        self.current.as_mut()
    }

    pub fn shape_count(&self) -> usize {
        self.shape_count
    }

    pub fn draw_commands(&self, bbox: [Point; 2]) -> impl Iterator<Item=DrawCommand> + '_ {
        let mut current = Vec::new();

        if let Some(shape) = self.current.as_ref() {
            current.push(shape.draw_commands());
        }

        let bbox = AABB::from_corners(bbox[0].to_a(), bbox[1].to_a());

        self.shapes.locate_in_envelope_intersecting(&bbox).map(|shape| shape.draw_commands()).chain(current)
    }

    /// Gets the bounding box that surrounds every shape drawn or returns None
    /// if there are no shapes
    pub fn get_bounds(&self) -> Option<[Point; 2]> {
        if self.shape_count() == 0 {
            return None;
        }

        Some(self.shapes.iter().map(|shape| {
            shape.envelope()
        }).fold([Point::new(f64::MAX, f64::MAX), Point::new(f64::MIN, f64::MIN)], |acc, cur| {
            let lower = Point::from(cur.lower());
            let upper = Point::from(cur.upper());

            [
                Point::new(
                    if lower.x < acc[0].x { lower.x } else { acc[0].x },
                    if lower.y < acc[0].y { lower.y } else { acc[0].y },
                ),
                Point::new(
                    if upper.x > acc[1].x { upper.x } else { acc[1].x },
                    if upper.y > acc[1].y { upper.y } else { acc[1].y },
                ),
            ]
        }))
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
    use super::Storage;

    use crate::shape::{Shape, ShapeType, ShapeId, Line, Rectangle, Ellipse};
    use crate::color::Color;
    use crate::draw_commands::DrawCommand;
    use crate::point::Point;

    #[test]
    fn test_add_shapes_at_zoom() {
        let mut storage = Storage::new();
        let shapes: Vec<Shape> = vec![
            Line::new(Color::red(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(1), 1.0),
            Rectangle::new(Color::green(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(2)),
            Ellipse::new(Color::blue(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(3)),
        ];
        let ids: Vec<_> = shapes.into_iter().map(|shape| {
            storage.add(shape)
        }).collect();

        storage.flush();

        assert_eq!(storage.shape_count(), 3);

        assert_eq!(ids[0], ShapeId::from(1));
        assert_eq!(ids[1], ShapeId::from(2));
        assert_eq!(ids[2], ShapeId::from(3));
    }

    #[test]
    fn test_last_mut() {
        let mut storage = Storage::new();

        assert!(storage.last_mut().is_none());

        storage.add(Line::new(Color::blue(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(1), 1.0));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Line);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));

        storage.add(Rectangle::new(Color::blue(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(2)));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Rectangle);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));

        storage.add(Ellipse::new(Color::blue(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(2)));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Ellipse);

        last_shape.handle(Point::new(0.0, 0.0));
        last_shape.handle(Point::new(1.0, 0.0));
    }

    #[test]
    fn test_flush() {
        let mut storage = Storage::new();

        assert!(storage.last_mut().is_none());

        storage.add(Line::new(Color::blue(), [Point::new(0.0, 0.0), Point::new(0.0, 0.0)], ShapeId::from(1), 1.0));

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
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let id_to_remove = ShapeId::from(1);
        let id_to_interfere = ShapeId::from(2);

        let obj_to_remove = Line::new(Color::green(), [Point::new(0.0, 0.0), Point::new(1.0, 1.0)], id_to_remove, 1.0);
        let obj_to_interfere = Line::new(Color::blue(), [Point::new(1.0, 0.0), Point::new(0.0, 1.0)], id_to_interfere, 1.0);

        let id = storage.add(obj_to_remove);
        storage.add(obj_to_interfere);

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn test_remove_by_id_2() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let id_to_remove = ShapeId::from(1);
        let id_to_interfere = ShapeId::from(2);

        let obj_to_remove = Line::new(Color::green(), [Point::new(0.0, 0.0), Point::new(1.0, 1.0)], id_to_remove, 1.0);
        let obj_to_interfere = Line::new(Color::blue(), [Point::new(1.0, 0.0), Point::new(0.0, 1.0)], id_to_interfere, 1.0);

        storage.add(obj_to_interfere);
        let id = storage.add(obj_to_remove);

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn test_iter_by_bounds() {
        let mut storage = Storage::new();

        let shapes = vec![
            // in green the ones that go
            Rectangle::new(Color::green(), [Point::new(-1.1, -1.1), Point::new(1.1, 1.1)], ShapeId::from(1)),
            Rectangle::new(Color::green(), [Point::new(-1.5, 0.5), Point::new(-0.5, 1.5)], ShapeId::from(2)),
            Rectangle::new(Color::green(), [Point::new(0.25, -0.5), Point::new(0.5, -0.25)], ShapeId::from(3)),

            // in red the ones that dont
            Rectangle::new(Color::red(), [Point::new(20.0, 20.0), Point::new(20.0, 20.0)], ShapeId::from(4)),
        ];

        for shape in shapes.into_iter() {
            storage.add(shape);
        }

        storage.flush();

        let buffer: Vec<_> = storage.draw_commands([Point::new(-1.0, -1.0), Point::new(1.0, 1.0)]).collect();

        assert_eq!(buffer.len(), 3);

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::green(),
                corner_1: Point::new(-1.1, -1.1),
                corner_2: Point::new(1.1, 1.1),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::green(),
                corner_1: Point::new(-1.5, 0.5),
                corner_2: Point::new(-0.5, 1.5),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Rectangle {
                color: Color::green(),
                corner_1: Point::new(0.25, -0.5),
                corner_2: Point::new(0.5, -0.25),
            }
        ));

        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_last_shape() {
        let mut storage = Storage::new();

        // A rectangle of 2x2 in zoom 0 looks like one of 1x1 when looked from zoom 0
        storage.add(Rectangle::new(Color::red(), [Point::new(1.0, 1.0), Point::new(3.0, 3.0)], ShapeId::from(2)));

        // only the previous shape is in permanent storage
        storage.flush();

        // A rectangle of 2x2 in zoom 0 aournd the oriign
        storage.add(Rectangle::new(Color::blue(), [Point::new(-1.0, -1.0), Point::new(1.0, 1.0)], ShapeId::from(1)));

        let buffer: Vec<_> = storage.draw_commands([Point::new(-2.0, -2.0), Point::new(4.0, 4.0)]).collect();

        assert_eq!(buffer.len(), 2);

        assert!(buffer.contains(&DrawCommand::Rectangle{
            color: Color::red(),
            corner_1: Point::new(1.0, 1.0),
            corner_2: Point::new(3.0, 3.0),
        }));
        assert!(buffer.contains(&DrawCommand::Rectangle{
            color: Color::blue(),
            corner_1: Point::new(-1.0, -1.0),
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
        let mut storage = Storage::new();

        storage.add(Rectangle::new(Color::blue(), [Point::new(-5.0, -5.0), Point::new(1.0, 1.0)], ShapeId::from(1)));
        storage.add(Rectangle::new(Color::blue(), [Point::new(5.0, 5.0), Point::new(1.0, 1.0)], ShapeId::from(1)));

        storage.flush();

        assert_eq!(storage.get_bounds(), Some([Point::new(-5.0, -5.0), Point::new(5.0, 5.0)]));
    }
}
