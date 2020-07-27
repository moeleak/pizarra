use std::collections::HashMap;
use std::f64;
use std::cell::RefCell;

use rstar::{RTree, RTreeObject, AABB, SelectionFunction, Envelope};

use super::shape::{Shape, ShapeId, ShapeTrait};
use super::draw_commands::DrawCommand;
use crate::point::Point;

impl RTreeObject for Shape {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let [p1, p2] = self.bbox();

        AABB::from_corners(p1, p2)
    }
}

/// Handles delete from the storage by Id
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

/// Handles delete from the storage by center and radius
struct CenterAndRadiusSelection {
    center: Point,
    radius: f64,
}

impl CenterAndRadiusSelection {
    fn new(center: Point, radius: f64) -> CenterAndRadiusSelection {
        CenterAndRadiusSelection {
            center, radius,
        }
    }
}

impl SelectionFunction<Shape> for CenterAndRadiusSelection {
    fn should_unpack_parent(&self, envelope: &<Shape as RTreeObject>::Envelope) -> bool {
        let bbox = AABB::from_corners(
            [self.center.x - self.radius, self.center.y - self.radius],
            [self.center.x + self.radius, self.center.y + self.radius],
        );

        envelope.intersects(&bbox)
    }

    fn should_unpack_leaf(&self, leaf: &Shape) -> bool {
        leaf.intersects_circle(self.center, self.radius)
    }
}

struct DrawCommandCache {
    commands: Vec<DrawCommand>,
    bbox: [Point; 2],
}

impl DrawCommandCache {
    fn add(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }
}

/// This struct handles storage and fast retrieval of shapes.
pub struct Storage {
    ids: HashMap<ShapeId, AABB<[f64; 2]>>,
    shapes: RTree<Shape>,
    current: Option<Shape>,
    next_id: ShapeId,
    next_z_index: usize,
    cache: RefCell<Option<DrawCommandCache>>,
}

/// A storage struct that organizes shapes by their zoom level and allows for
/// fast queries given a zoom and a bbox.
impl Storage {
    pub fn new() -> Storage {
        Storage {
            shapes: RTree::new(),
            ids: HashMap::new(),
            current: None,
            next_id: ShapeId::from(1),
            next_z_index: 1,
            cache: RefCell::new(None),
        }
    }

    pub fn nex_id(&mut self) -> ShapeId {
        let id = self.next_id;
        self.next_id = id.next();
        id
    }

    pub fn next_z_index(&mut self) -> usize {
        let index = self.next_z_index;
        self.next_z_index = index + 1;
        index
    }

    /// Adds a new shape to the storage
    pub fn add(&mut self, shape: Box<dyn ShapeTrait>) -> ShapeId {
        // The previous shape has been finished, move it to persistent storage
        self.flush();

        let id = self.nex_id();

        self.current = Some(Shape::from_shape(shape, id, self.next_z_index()));

        id
    }

    /// Restores a deleted shape
    pub fn restore(&mut self, shape: Shape) -> ShapeId {
        let shape_id = shape.id();

        self.ids.insert(shape_id, shape.envelope());
        self.shapes.insert(shape);

        // Invalidate cache
        self.cache.swap(&RefCell::new(None));

        shape_id
    }

    /// moves the last inserted shape to persistent storage. It will no longer
    /// be accesible through .last_mut()
    pub fn flush(&mut self) -> Option<ShapeId> {
        let old_current = self.current.take();

        if let Some(old_shape) = old_current {
            let old_id = old_shape.id();
            let draw_command = old_shape.draw_commands();

            self.ids.insert(old_id, old_shape.envelope());
            self.shapes.insert(old_shape);

            if let Some(cache) = self.cache.borrow_mut().as_mut() {
                cache.add(draw_command);
            }

            Some(old_id)
        } else {
            None
        }
    }

    /// Deletes a shape from this storage given its id if such id exists
    pub fn remove(&mut self, id: ShapeId) -> Option<Shape> {
        if let Some(bbox) = self.ids.get(&id) {
            let maybe_shape = self.shapes.remove_with_selection_function(BBoxIdSelection::new(*bbox, id));

            // Invalidate cache
            self.cache.swap(&RefCell::new(None));

            maybe_shape
        } else {
            None
        }
    }

    /// Returns the id of a shape intersecting the circle described by center
    /// and radius, if any.
    pub fn query_circle(&self, center: Point, radius: f64) -> Option<ShapeId> {
        self.shapes.locate_with_selection_function(CenterAndRadiusSelection::new(center, radius)).next().map(|shape| shape.id())
    }

    /// gets a mutable reference to the last inserted shape for modifying
    pub fn last_mut(&mut self) -> Option<&mut Shape> {
        self.current.as_mut()
    }

    /// Returns the number of shapes in this storage
    pub fn shape_count(&self) -> usize {
        self.shapes.size()
    }

    /// returns the draw commands necessary to paint the portion of the screen
    /// indicated by bbox.
    fn query_commands(&self, bbox: [Point; 2]) -> Vec<DrawCommand> {
        let bbox = AABB::from_corners(bbox[0].to_a(), bbox[1].to_a());
        let mut commands: Vec<_> = self
            .shapes
            .locate_in_envelope_intersecting(&bbox)
            .map(|shape| (shape.z_index(), shape.draw_commands()))
            .collect();

        commands.sort_unstable_by_key(|(i, _dc)| *i);

        commands.into_iter().map(|(_i, dc)| dc).collect()
    }

    /// gets all the draw commands necessary to paint the portion of the screen
    /// indicated by bbox. Commands are cached to improve the performance of
    /// continuously drawing on the same portion of the screen
    pub fn draw_commands(&self, bbox: [Point; 2]) -> Vec<DrawCommand> {
        let mut borrow = self.cache.borrow_mut();

        let mut commands = match borrow.as_ref() {
            Some(cache) if cache.bbox == bbox => {
                cache.commands.clone()
            },
            _ => {
                borrow.replace(DrawCommandCache {
                    commands: self.query_commands(bbox),
                    bbox,
                });

                borrow.as_ref().unwrap().commands.clone()
            }
        };

        if let Some(shape) = self.current.as_ref() {
            commands.push(shape.draw_commands());
        }

        commands
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
    use std::rc::Rc;

    use super::Storage;

    use crate::shape::{ShapeType, ShapeTrait, ShapeId, Line, Rectangle, Ellipse};
    use crate::color::Color;
    use crate::draw_commands::DrawCommand;
    use crate::point::Point;

    #[test]
    fn test_add_shapes_at_zoom() {
        let mut storage = Storage::new();
        let shapes: Vec<Box<dyn ShapeTrait>> = vec![
            Box::new(Line::new(Color::red(), Point::new(0.0, 0.0), 1.0)),
            Box::new(Rectangle::new(Color::green(), Point::new(0.0, 0.0), 1.0)),
            Box::new(Ellipse::new(Color::blue(), Point::new(0.0, 0.0), 1.0)),
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

        storage.add(Box::new(Line::new(Color::blue(), Point::new(0.0, 0.0), 1.0)));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Line);

        last_shape.handle_mouse_moved(Point::new(0.0, 0.0));
        last_shape.handle_mouse_moved(Point::new(1.0, 0.0));

        storage.add(Box::new(Rectangle::from_corners([Point::new(0.0, 0.0), Point::new(0.0, 0.0)])));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Rectangle);

        last_shape.handle_mouse_moved(Point::new(0.0, 0.0));
        last_shape.handle_mouse_moved(Point::new(1.0, 0.0));

        storage.add(Box::new(Ellipse::new(Color::blue(), Point::new(0.0, 0.0), 1.0)));

        let last_shape = storage.last_mut().unwrap();

        assert_eq!(last_shape.shape_type(), ShapeType::Ellipse);

        last_shape.handle_mouse_moved(Point::new(0.0, 0.0));
        last_shape.handle_mouse_moved(Point::new(1.0, 0.0));
    }

    #[test]
    fn test_flush() {
        let mut storage = Storage::new();

        assert!(storage.last_mut().is_none());

        storage.add(Box::new(Line::new(Color::blue(), Point::new(0.0, 0.0), 1.0)));

        assert!(storage.last_mut().is_some());

        storage.flush();

        assert!(storage.last_mut().is_none());
    }

    #[test]
    fn test_remove_line() {
        let mut storage = Storage::new();

        let mut line = Line::new(Color::green(), Point::new(0.0, -1.0), 1.0);

        line.handle_mouse_moved(Point::new(0.0, 0.0));
        line.handle_mouse_moved(Point::new(1.0, 0.0));
        line.handle_mouse_moved(Point::new(1.0, 1.0));
        line.handle_mouse_moved(Point::new(0.0, 1.0));

        storage.add(Box::new(line));
        storage.flush();

        assert!(storage.query_circle(Point::new(0.5, -0.5), 0.5).is_none());
        assert!(storage.query_circle(Point::new(0.5, 0.5), 0.5).is_none());

        assert_eq!(storage.query_circle(Point::new(-0.25, -1.25), 0.5).unwrap(), ShapeId::from(1));
    }

    #[test]
    fn test_remove_by_id() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let obj_to_remove = Box::new(Line::new(Color::green(), Point::new(1.0, 1.0), 1.0));
        let obj_to_interfere = Box::new(Line::new(Color::blue(), Point::new(0.0, 1.0), 1.0));

        let id_to_remove = storage.add(obj_to_remove);
        let _id_to_interfere = storage.add(obj_to_interfere);

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id_to_remove).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn test_remove_by_id_2() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let obj_to_remove = Box::new(Line::new(Color::green(), Point::new(1.0, 1.0), 1.0));
        let obj_to_interfere = Box::new(Line::new(Color::blue(), Point::new(0.0, 1.0), 1.0));

        let _id_to_interfere = storage.add(obj_to_interfere);
        let id_to_remove = storage.add(obj_to_remove);

        storage.flush();

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id_to_remove).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn incremental_drawing() {
        let mut storage = Storage::new();
        let bbox = [Point::new(-40.0, -40.0), Point::new(40.0, 40.0)];

        storage.add(Box::new(Line::new(Color::green(), Point::new(0.0, 0.0), 4.0)));

        storage.last_mut().unwrap().handle_mouse_moved(Point::new(1.0, 0.0));

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)]),
            thickness: 4.0,
        }]);

        storage.last_mut().unwrap().handle_mouse_moved(Point::new(2.0, 0.0));

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0)]),
            thickness: 4.0,
        }]);

        storage.last_mut().unwrap().handle_button_released(Point::new(3.0, 0.0));

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0), Point::new(3.0, 0.0)]),
            thickness: 4.0,
        }]);

        storage.flush();

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0), Point::new(3.0, 0.0)]),
            thickness: 4.0,
        }]);
    }

    #[test]
    fn test_erase_invalidates_cache() {
        let mut storage = Storage::new();
        let bbox = [Point::new(-40.0, -40.0), Point::new(40.0, 40.0)];

        storage.add(Box::new(Line::with_params(Color::green(), vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)], 4.0)));
        storage.add(Box::new(Line::with_params(Color::green(), vec![Point::new(0.0, 1.0), Point::new(1.0, 1.0)], 4.0)));

        let id = storage.flush().unwrap();

        storage.remove(id);

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)]),
            thickness: 4.0,
        }]);
    }

    #[test]
    fn test_iter_by_bounds() {
        let mut storage = Storage::new();

        let shapes = vec![
            // in green the ones that go
            Box::new(Rectangle::from_corners([Point::new(-1.1, -1.1), Point::new(1.1, 1.1)])),
            Box::new(Rectangle::from_corners([Point::new(-1.5, 0.5), Point::new(-0.5, 1.5)])),
            Box::new(Rectangle::from_corners([Point::new(0.25, -0.5), Point::new(0.5, -0.25)])),

            // in red the ones that dont
            Box::new(Rectangle::from_corners([Point::new(20.0, 20.0), Point::new(20.0, 20.0)])),
        ];

        for shape in shapes.into_iter() {
            storage.add(shape);
        }

        storage.flush();

        let buffer = storage.draw_commands([Point::new(-1.0, -1.0), Point::new(1.0, 1.0)]);

        assert_eq!(buffer.len(), 3);

        assert!(buffer.contains(
            &DrawCommand::Line {
                thickness: 4.0,
                color: Color::green(),
                line: Rc::new(vec![
                    Point::new(0.25, -0.5),
                    Point::new(0.25, -0.25),
                    Point::new(0.5, -0.25),
                    Point::new(0.5, -0.5),
                    Point::new(0.25, -0.5),
                ]),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Line {
                thickness: 4.0,
                color: Color::green(),
                line: Rc::new(vec![
                    Point::new(-1.5, 0.5),
                    Point::new(-1.5, 1.5),
                    Point::new(-0.5, 1.5),
                    Point::new(-0.5, 0.5),
                    Point::new(-1.5, 0.5),
                ]),
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Line {
                thickness: 4.0,
                color: Color::green(),
                line: Rc::new(vec![
                    Point::new(-1.1, -1.1),
                    Point::new(-1.1, 1.1),
                    Point::new(1.1, 1.1),
                    Point::new(1.1, -1.1),
                    Point::new(-1.1, -1.1),
                ]),
            }
        ));

        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_last_shape() {
        let mut storage = Storage::new();

        // A rectangle of 2x2 in zoom 0 looks like one of 1x1 when looked from zoom 0
        storage.add(Box::new(Rectangle::from_corners([Point::new(1.0, 1.0), Point::new(3.0, 3.0)])));

        // only the previous shape is in permanent storage
        storage.flush();

        // A rectangle of 2x2 in zoom 0 aournd the oriign
        storage.add(Box::new(Rectangle::from_corners([Point::new(-1.0, -1.0), Point::new(1.0, 1.0)])));

        let buffer = storage.draw_commands([Point::new(-2.0, -2.0), Point::new(4.0, 4.0)]);

        assert_eq!(buffer.len(), 2);

        assert!(buffer.contains(&DrawCommand::Line{
            thickness: 4.0,
            color: Color::green(),
            line: Rc::new(vec![
                Point::new(1.0, 1.0),
                Point::new(1.0, 3.0),
                Point::new(3.0, 3.0),
                Point::new(3.0, 1.0),
                Point::new(1.0, 1.0),
            ]),
        }));
        assert!(buffer.contains(&DrawCommand::Line{
            thickness: 4.0,
            color: Color::green(),
            line: Rc::new(vec![
                Point::new(-1.0, -1.0),
                Point::new(-1.0, 1.0),
                Point::new(1.0, 1.0),
                Point::new(1.0, -1.0),
                Point::new(-1.0, -1.0),
            ]),
        }));
    }

    #[test]
    fn test_get_bounds() {
        let mut storage = Storage::new();

        storage.add(Box::new(Rectangle::from_corners([Point::new(-5.0, -5.0), Point::new(1.0, 1.0)])));
        storage.add(Box::new(Rectangle::from_corners([Point::new(5.0, 5.0), Point::new(1.0, 1.0)])));

        storage.flush();

        assert_eq!(storage.get_bounds(), Some([Point::new(-5.0, -5.0), Point::new(5.0, 5.0)]));
    }

    #[test]
    fn test_draw_oder_is_respected() {
        let mut storage = Storage::new();

        storage.add(Box::new(Line::with_params(Color::red(), vec![Point::new(0.0, 0.0), Point::new(0.0, 2.0)], 4.0)));
        storage.add(Box::new(Line::with_params(Color::green(), vec![Point::new(0.0, 0.0), Point::new(0.0, 1.0)], 4.0)));

        storage.flush();

        let commands = storage.draw_commands([Point::new(-1.0, -1.0), Point::new(1.0, 3.0)]);

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0], DrawCommand::Line {
            color: Color::red(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(0.0, 2.0)]),
            thickness: 4.0,
        });
        assert_eq!(commands[1], DrawCommand::Line {
            color: Color::green(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(0.0, 1.0)]),
            thickness: 4.0,
        });
    }
}
