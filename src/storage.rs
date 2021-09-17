use std::collections::HashMap;
use std::f64;

use rstar::{RTree, RTreeObject, AABB, SelectionFunction, Envelope};

use crate::point::{Point, Vec2DDrawing};

use super::shape::{Shape, ShapeId, ShapeStored};
use super::draw_commands::DrawCommand;

impl RTreeObject for Shape {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let [p1, p2] = self.bbox();

        AABB::from_corners(p1.to_a(), p2.to_a())
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
    center: Vec2DDrawing,
    radius: f64,
}

impl CenterAndRadiusSelection {
    fn new(center: Vec2DDrawing, radius: f64) -> CenterAndRadiusSelection {
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

/// This struct handles storage and fast retrieval of shapes.
pub struct Storage {
    /// used to easily find a shape that will be deleted by id, because the
    /// rstar tree knows how to find bboxes easily but not ids.
    ids: HashMap<ShapeId, AABB<[f64; 2]>>,

    /// The rstart tree that backs this storage.
    shapes: RTree<Shape>,

    /// The next id available.
    next_id: ShapeId,

    /// the next z-index available.
    next_z_index: usize,
}

/// A storage struct that organizes shapes by their zoom level and allows for
/// fast queries given a zoom and a bbox.
impl Storage {
    pub fn new() -> Storage {
        Storage {
            shapes: RTree::new(),
            ids: HashMap::new(),
            next_id: ShapeId::from(1),
            next_z_index: 1,
        }
    }

    fn nex_id(&mut self) -> ShapeId {
        let id = self.next_id;
        self.next_id = id.next();
        id
    }

    fn next_z_index(&mut self) -> usize {
        let index = self.next_z_index;
        self.next_z_index = index + 1;
        index
    }

    /// Adds a new shape to the storage
    pub fn add(&mut self, shape: Box<dyn ShapeStored>) -> ShapeId {
        // The previous shape has been finished, move it to persistent storage
        let id = self.nex_id();
        let z_index = self.next_z_index();
        let shape = Shape::from_shape(shape, id, z_index);

        self.ids.insert(id, shape.envelope());
        self.shapes.insert(shape);

        id
    }

    /// Restores a deleted shape
    pub fn restore(&mut self, shape: Shape) -> ShapeId {
        let shape_id = shape.id();

        self.ids.insert(shape_id, shape.envelope());
        self.shapes.insert(shape);

        shape_id
    }

    /// Deletes a shape from this storage given its id if such id exists
    pub fn remove(&mut self, id: ShapeId) -> Option<Shape> {
        if let Some(bbox) = self.ids.get(&id) {
            let maybe_shape = self.shapes.remove_with_selection_function(BBoxIdSelection::new(*bbox, id));

            maybe_shape
        } else {
            None
        }
    }

    /// Returns the id of a shape intersecting the circle described by center
    /// and radius, if any.
    pub fn query_circle(&self, center: Vec2DDrawing, radius: f64) -> Option<ShapeId> {
        self.shapes.locate_with_selection_function(CenterAndRadiusSelection::new(center, radius)).next().map(|shape| shape.id())
    }

    /// Returns the number of shapes in this storage
    pub fn shape_count(&self) -> usize {
        self.shapes.size()
    }

    /// returns the draw commands necessary to paint the portion of the screen
    /// indicated by bbox.
    fn query_commands(&self, bbox: [Vec2DDrawing; 2]) -> Vec<DrawCommand> {
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
    pub fn draw_commands(&self, bbox: [Vec2DDrawing; 2]) -> Vec<DrawCommand> {
        self.query_commands(bbox)
    }

    /// Gets the bounding box that surrounds every shape drawn or returns None
    /// if there are no shapes
    pub fn get_bounds(&self) -> Option<[Vec2DDrawing; 2]> {
        if self.shape_count() == 0 {
            return None;
        }

        Some(self.shapes.iter().map(|shape| {
            shape.envelope()
        }).fold([Vec2DDrawing::new(f64::MAX, f64::MAX), Vec2DDrawing::new(f64::MIN, f64::MIN)], |acc, cur| {
            let lower = Vec2DDrawing::from(cur.lower());
            let upper = Vec2DDrawing::from(cur.upper());

            [
                Vec2DDrawing::new(
                    if lower.x < acc[0].x { lower.x } else { acc[0].x },
                    if lower.y < acc[0].y { lower.y } else { acc[0].y },
                ),
                Vec2DDrawing::new(
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
    use super::*;

    use crate::shape::{
        ShapeId,
        path::PathCommand,
        circle::Circle, path::Path, rectangle::Rectangle,
    };
    use crate::color::Color;
    use crate::draw_commands::DrawCommand;
    use crate::point::Vec2DDrawing;

    #[test]
    fn add_shapes_at_zoom() {
        let mut storage = Storage::new();
        let shapes: Vec<Box<dyn ShapeStored>> = vec![
            Box::new(Circle::from_parts(Vec2DDrawing::new(0.0, 0.0), 1.0, 1.0, Color::green())),
            Box::new(Path::from_parts(vec![], Color::red(), 1.0)),
            Box::new(Rectangle::from_parts([Vec2DDrawing::new(0.0, 0.0); 4], Color::green(), 1.0)),
        ];
        let ids: Vec<_> = shapes.into_iter().map(|shape| {
            storage.add(shape)
        }).collect();

        assert_eq!(storage.shape_count(), 3);

        assert_eq!(ids[0], ShapeId::from(1));
        assert_eq!(ids[1], ShapeId::from(2));
        assert_eq!(ids[2], ShapeId::from(3));
    }

    #[test]
    fn remove_line() {
        let mut storage = Storage::new();
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(0.0, -1.0)),
            PathCommand::LineTo(Vec2DDrawing::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(1.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(1.0, 1.0)),
            PathCommand::LineTo(Vec2DDrawing::new(0.0, 1.0)),
        ], Color::green(), 1.0);

        storage.add(Box::new(line));

        assert!(storage.query_circle(Vec2DDrawing::new(0.5, -0.5), 0.5).is_none());
        assert!(storage.query_circle(Vec2DDrawing::new(0.5, 0.5), 0.5).is_none());

        assert_eq!(storage.query_circle(Vec2DDrawing::new(-0.25, -1.25), 0.5).unwrap(), ShapeId::from(1));
    }

    #[test]
    fn remove_by_id() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let obj_to_remove = Box::new(Path::new());
        let obj_to_interfere = Box::new(Path::new());

        let id_to_remove = storage.add(obj_to_remove);
        let _id_to_interfere = storage.add(obj_to_interfere);

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id_to_remove).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn remove_by_id_2() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let obj_to_remove = Box::new(Path::new());
        let obj_to_interfere = Box::new(Path::new());

        let _id_to_interfere = storage.add(obj_to_interfere);
        let id_to_remove = storage.add(obj_to_remove);

        assert_eq!(storage.shape_count(), 2);

        let data = storage.remove(id_to_remove).unwrap();

        assert_eq!(storage.shape_count(), 1);

        assert_eq!(data.id(), id_to_remove);
    }

    #[test]
    fn erase_invalidates_cache() {
        let mut storage = Storage::new();
        let bbox = [Vec2DDrawing::new(-40.0, -40.0), Vec2DDrawing::new(40.0, 40.0)];

        storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(1.0, 0.0)),
        ], Color::green(), 4.0)));
        let id = storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(0.0, 1.0)),
            PathCommand::LineTo(Vec2DDrawing::new(1.0, 1.0)),
        ], Color::green(), 4.0)));

        storage.remove(id);

        assert_eq!(storage.draw_commands(bbox), vec![DrawCommand::Path {
            color: Color::green(),
            commands: vec![
                PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2DDrawing::new(1.0, 0.0)),
            ],
            thickness: 4.0,
        }]);
    }

    #[test]
    fn iter_by_bounds() {
        let mut storage = Storage::new();

        let shapes = vec![
            // in green the ones that go
            Box::new(Rectangle::from_parts([
                Vec2DDrawing::new(-1.1, -1.1),
                Vec2DDrawing::new(-1.1, 1.1),
                Vec2DDrawing::new(1.1, 1.1),
                Vec2DDrawing::new(1.1, -1.1),
            ], Default::default(), 1.0)),
            Box::new(Rectangle::from_parts([
                Vec2DDrawing::new(-1.5, 0.5),
                Vec2DDrawing::new(-1.5, 0.5),
                Vec2DDrawing::new(-0.5, 1.5),
                Vec2DDrawing::new(-0.5, 1.5),
            ], Default::default(), 1.0)),
            Box::new(Rectangle::from_parts([
                Vec2DDrawing::new(0.25, -0.5),
                Vec2DDrawing::new(0.25, -0.5),
                Vec2DDrawing::new(0.5, -0.25),
                Vec2DDrawing::new(0.5, -0.25),
            ], Default::default(), 1.0)),

            // in red the ones that dont
            Box::new(Rectangle::from_parts([
                Vec2DDrawing::new(20.0, 20.0),
                Vec2DDrawing::new(20.0, 20.0),
                Vec2DDrawing::new(20.0, 20.0),
                Vec2DDrawing::new(20.0, 20.0),
            ], Default::default(), 1.0)),
        ];

        for shape in shapes.into_iter() {
            storage.add(shape);
        }

        let buffer = storage.draw_commands([Vec2DDrawing::new(-1.0, -1.0), Vec2DDrawing::new(1.0, 1.0)]);

        assert_eq!(buffer.len(), 3);

        assert!(buffer.contains(
            &DrawCommand::Path {
                thickness: 4.0,
                color: Color::green(),
                commands: vec![
                    PathCommand::MoveTo(Vec2DDrawing::new(0.25, -0.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(0.25, -0.25)),
                    PathCommand::LineTo(Vec2DDrawing::new(0.5, -0.25)),
                    PathCommand::LineTo(Vec2DDrawing::new(0.5, -0.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(0.25, -0.5)),
                ],
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Path {
                thickness: 4.0,
                color: Color::green(),
                commands: vec![
                    PathCommand::MoveTo(Vec2DDrawing::new(-1.5, 0.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(-1.5, 1.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(-0.5, 1.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(-0.5, 0.5)),
                    PathCommand::LineTo(Vec2DDrawing::new(-1.5, 0.5)),
                ],
            }
        ));

        assert!(buffer.contains(
            &DrawCommand::Path {
                thickness: 4.0,
                color: Color::green(),
                commands: vec![
                    PathCommand::MoveTo(Vec2DDrawing::new(-1.1, -1.1)),
                    PathCommand::LineTo(Vec2DDrawing::new(-1.1, 1.1)),
                    PathCommand::LineTo(Vec2DDrawing::new(1.1, 1.1)),
                    PathCommand::LineTo(Vec2DDrawing::new(1.1, -1.1)),
                    PathCommand::LineTo(Vec2DDrawing::new(-1.1, -1.1)),
                ],
            }
        ));

        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn last_shape() {
        let mut storage = Storage::new();

        // A rectangle of 2x2 in zoom 0 looks like one of 1x1 when looked from zoom 0
        storage.add(Box::new(Rectangle::from_parts([
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(3.0, 1.0),
            Vec2DDrawing::new(3.0, 3.0),
            Vec2DDrawing::new(1.0, 3.0),
        ], Default::default(), 1.0)));

        // A rectangle of 2x2 in zoom 0 aournd the oriign
        storage.add(Box::new(Rectangle::from_parts([
            Vec2DDrawing::new(-1.0, -1.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
        ], Default::default(), 1.0)));

        let buffer = storage.draw_commands([Vec2DDrawing::new(-2.0, -2.0), Vec2DDrawing::new(4.0, 4.0)]);

        assert_eq!(buffer.len(), 1);

        assert!(buffer.contains(&DrawCommand::Path{
            thickness: 4.0,
            color: Color::green(),
            commands: vec![
                PathCommand::MoveTo(Vec2DDrawing::new(1.0, 1.0)),
                PathCommand::LineTo(Vec2DDrawing::new(1.0, 3.0)),
                PathCommand::LineTo(Vec2DDrawing::new(3.0, 3.0)),
                PathCommand::LineTo(Vec2DDrawing::new(3.0, 1.0)),
                PathCommand::LineTo(Vec2DDrawing::new(1.0, 1.0)),
            ],
        }));
    }

    #[test]
    fn get_bounds() {
        let mut storage = Storage::new();

        storage.add(Box::new(Rectangle::from_parts([
            Vec2DDrawing::new(-5.0, -5.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
        ], Default::default(), 1.0)));
        storage.add(Box::new(Rectangle::from_parts([
            Vec2DDrawing::new(5.0, 5.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
            Vec2DDrawing::new(1.0, 1.0),
        ], Default::default(), 1.0)));

        assert_eq!(storage.get_bounds(), Some([Vec2DDrawing::new(-5.0, -5.0), Vec2DDrawing::new(5.0, 5.0)]));
    }

    #[test]
    fn draw_oder_is_respected() {
        let mut storage = Storage::new();

        storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(0.0, 2.0)),
        ], Color::red(), 4.0)));
        storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(0.0, 1.0)),
        ],Color::green(),  4.0)));

        let commands = storage.draw_commands([Vec2DDrawing::new(-1.0, -1.0), Vec2DDrawing::new(1.0, 3.0)]);

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].color(), Color::red());
        assert_eq!(commands[1].color(), Color::green());
    }
}
