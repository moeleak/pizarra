use std::collections::HashMap;
use std::f64;

use rstar::{Envelope, Point, RTree, RTreeObject, SelectionFunction, AABB};

use crate::point::{Vec2D, WorldUnit};

use super::draw_commands::DrawCommand;
use super::shape::{Shape, ShapeId, ShapeStored};

impl Point for Vec2D<WorldUnit> {
    type Scalar = WorldUnit;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Vec2D {
            x: generator(0),
            y: generator(1),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

impl RTreeObject for Shape {
    type Envelope = AABB<Vec2D<WorldUnit>>;

    fn envelope(&self) -> Self::Envelope {
        let [p1, p2] = self.bbox();

        AABB::from_corners(p1, p2)
    }
}

/// Handles delete from the storage by Id
struct BBoxIdSelection {
    bbox: AABB<Vec2D<WorldUnit>>,
    id: ShapeId,
}

impl BBoxIdSelection {
    fn new(bbox: AABB<Vec2D<WorldUnit>>, id: ShapeId) -> BBoxIdSelection {
        BBoxIdSelection { bbox, id }
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
    center: Vec2D<WorldUnit>,
    radius: WorldUnit,
}

impl CenterAndRadiusSelection {
    fn new(center: Vec2D<WorldUnit>, radius: WorldUnit) -> CenterAndRadiusSelection {
        CenterAndRadiusSelection { center, radius }
    }
}

impl SelectionFunction<Shape> for CenterAndRadiusSelection {
    fn should_unpack_parent(&self, envelope: &<Shape as RTreeObject>::Envelope) -> bool {
        let bbox = AABB::from_corners(
            self.center - Vec2D::new(self.radius, self.radius),
            self.center + Vec2D::new(self.radius, self.radius),
        );

        envelope.intersects(&bbox)
    }

    fn should_unpack_leaf(&self, leaf: &Shape) -> bool {
        leaf.intersects_circle(self.center, self.radius)
    }
}

/// This struct handles storage and fast retrieval of shapes.
#[derive(Debug)]
pub struct Storage {
    /// used to easily find a shape that will be deleted by id, because the
    /// rstar tree knows how to find bboxes easily but not ids.
    ids: HashMap<ShapeId, AABB<Vec2D<WorldUnit>>>,

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
            self.shapes
                .remove_with_selection_function(BBoxIdSelection::new(*bbox, id))
        } else {
            None
        }
    }

    /// Returns the id of a shape intersecting the circle described by center
    /// and radius, if any.
    pub fn remove_circle(
        &mut self,
        center: Vec2D<WorldUnit>,
        radius: WorldUnit,
    ) -> impl Iterator<Item = Shape> + '_ {
        self.shapes
            .drain_with_selection_function(CenterAndRadiusSelection::new(center, radius))
    }

    /// Returns the number of shapes in this storage
    pub fn shape_count(&self) -> usize {
        self.shapes.size()
    }

    /// returns the draw commands necessary to paint the portion of the screen
    /// indicated by bbox.
    fn query_commands(&self, bbox: [Vec2D<WorldUnit>; 2]) -> Vec<DrawCommand> {
        let bbox = AABB::from_corners(bbox[0], bbox[1]);
        let mut commands: Vec<_> = self
            .shapes
            .locate_in_envelope_intersecting(&bbox)
            .map(|shape| (shape.z_index(), shape.draw_commands()))
            .collect();

        commands.sort_unstable_by_key(|(i, _dc)| *i);

        commands.into_iter().map(|(_i, dc)| dc).collect()
    }

    /// Returns the shapes stored in this storage ordered by their index. It is
    /// used for rendering the shapes in the order they where drawn. Creates an
    /// allocation.
    pub fn shapes_by_index(&self) -> impl Iterator<Item = &dyn ShapeStored> {
        let mut shapes: Vec<_> = self.shapes.iter().collect();

        shapes.sort_unstable_by_key(|s| s.z_index());

        shapes.into_iter().map(|s| s.shape_impl())
    }

    /// gets all the draw commands necessary to paint the portion of the screen
    /// indicated by bbox. Commands are cached to improve the performance of
    /// continuously drawing on the same portion of the screen
    pub fn draw_commands(&self, bbox: [Vec2D<WorldUnit>; 2]) -> Vec<DrawCommand> {
        self.query_commands(bbox)
    }

    /// Gets the bounding box that surrounds every shape drawn or returns None
    /// if there are no shapes
    pub fn get_bounds(&self) -> Option<[Vec2D<WorldUnit>; 2]> {
        if self.shape_count() == 0 {
            return None;
        }

        Some(self.shapes.iter().map(|shape| shape.envelope()).fold(
            [
                Vec2D::new_world(f64::MAX, f64::MAX),
                Vec2D::new_world(f64::MIN, f64::MIN),
            ],
            |acc, cur| {
                let lower = cur.lower();
                let upper = cur.upper();

                [
                    Vec2D::new(
                        if lower.x < acc[0].x {
                            lower.x
                        } else {
                            acc[0].x
                        },
                        if lower.y < acc[0].y {
                            lower.y
                        } else {
                            acc[0].y
                        },
                    ),
                    Vec2D::new(
                        if upper.x > acc[1].x {
                            upper.x
                        } else {
                            acc[1].x
                        },
                        if upper.y > acc[1].y {
                            upper.y
                        } else {
                            acc[1].y
                        },
                    ),
                ]
            },
        ))
    }
}

impl Default for Storage {
    fn default() -> Storage {
        Storage::new()
    }
}

pub struct ShapeIterator<'a> {
    iterator: std::slice::Iter<'a, Shape>,
}

impl<'a> Iterator for ShapeIterator<'a> {
    type Item = &'a Shape;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::color::Color;
    use crate::draw_commands::DrawCommand;
    use crate::geom::Angle;
    use crate::path_command::PathCommand;
    use crate::shape::{
        stored::{ellipse::Ellipse, path::Path},
        ShapeId,
    };
    use crate::style::{Stroke, Style};

    use super::*;

    #[test]
    fn add_shapes_at_zoom() {
        let mut storage = Storage::new();
        let shapes: Vec<Box<dyn ShapeStored>> = vec![
            Box::new(Ellipse::from_parts(
                Vec2D::new_world(0.0, 0.0),
                1.0.into(),
                1.0.into(),
                Angle::from_radians(0.0),
                Default::default(),
            )),
            Box::new(Path::from_parts(
                vec![
                    PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                    PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                ],
                Default::default(),
            )),
        ];
        let ids: Vec<_> = shapes.into_iter().map(|shape| storage.add(shape)).collect();

        assert_eq!(storage.shape_count(), 2);

        assert_eq!(ids[0], ShapeId::from(1));
        assert_eq!(ids[1], ShapeId::from(2));
    }

    #[test]
    fn remove_line() {
        let mut storage = Storage::new();
        let line = Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, -1.0)),
                PathCommand::LineTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(0.0, 1.0)),
            ],
            Style {
                stroke: Some(Stroke {
                    size: 0.0.into(),
                    color: Default::default(),
                }),
                fill: None,
            },
        );

        storage.add(Box::new(line));

        assert_eq!(
            storage
                .remove_circle(Vec2D::new_world(0.5, -0.5), 0.5.into())
                .collect::<Vec<_>>()
                .len(),
            0
        );
        assert_eq!(
            storage
                .remove_circle(Vec2D::new_world(0.5, 0.5), 0.5.into())
                .collect::<Vec<_>>()
                .len(),
            0
        );

        assert!(storage
            .remove_circle(Vec2D::new_world(-0.25, -1.25), 0.5.into())
            .next()
            .is_some());
    }

    #[test]
    fn remove_by_id() {
        let mut storage = Storage::new();

        assert_eq!(storage.shape_count(), 0);

        let obj_to_remove = Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
            ],
            Default::default(),
        ));
        let obj_to_interfere = Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
            ],
            Default::default(),
        ));

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

        let obj_to_remove = Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
            ],
            Default::default(),
        ));
        let obj_to_interfere = Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
            ],
            Default::default(),
        ));

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
        let bbox = [Vec2D::new_world(-40.0, -40.0), Vec2D::new_world(40.0, 40.0)];

        storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
            ],
            Default::default(),
        )));
        let id = storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
            ],
            Default::default(),
        )));

        storage.remove(id);

        assert_eq!(
            storage.draw_commands(bbox),
            vec![DrawCommand::Path {
                style: Default::default(),
                commands: vec![
                    PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                    PathCommand::LineTo(Vec2D::new_world(1.0, 0.0)),
                ],
            }]
        );
    }

    #[test]
    fn iter_by_bounds() {
        let mut storage = Storage::new();

        let shapes = vec![
            // in green the ones that go
            Box::new(Path::from_parts(
                vec![
                    PathCommand::MoveTo(Vec2D::new_world(-1.1, -1.1)),
                    PathCommand::LineTo(Vec2D::new_world(-1.1, 1.1)),
                    PathCommand::LineTo(Vec2D::new_world(1.1, 1.1)),
                    PathCommand::LineTo(Vec2D::new_world(1.1, -1.1)),
                ],
                Default::default(),
            )),
            Box::new(Path::from_parts(
                vec![
                    PathCommand::MoveTo(Vec2D::new_world(-1.5, 0.5)),
                    PathCommand::LineTo(Vec2D::new_world(-1.5, 0.5)),
                    PathCommand::LineTo(Vec2D::new_world(-0.5, 1.5)),
                    PathCommand::LineTo(Vec2D::new_world(-0.5, 1.5)),
                ],
                Default::default(),
            )),
            Box::new(Path::from_parts(
                vec![
                    PathCommand::MoveTo(Vec2D::new_world(0.25, -0.5)),
                    PathCommand::LineTo(Vec2D::new_world(0.25, -0.5)),
                    PathCommand::LineTo(Vec2D::new_world(0.5, -0.25)),
                    PathCommand::LineTo(Vec2D::new_world(0.5, -0.25)),
                ],
                Default::default(),
            )),
            // in red the ones that dont
            Box::new(Path::from_parts(
                vec![
                    PathCommand::MoveTo(Vec2D::new_world(20.0, 20.0)),
                    PathCommand::LineTo(Vec2D::new_world(20.0, 20.0)),
                    PathCommand::LineTo(Vec2D::new_world(20.0, 20.0)),
                    PathCommand::LineTo(Vec2D::new_world(20.0, 20.0)),
                ],
                Style {
                    stroke: Some(Stroke {
                        color: Color::red(),
                        size: 1.0.into(),
                    }),
                    fill: None,
                },
            )),
        ];

        for shape in shapes.into_iter() {
            storage.add(shape);
        }

        let buffer =
            storage.draw_commands([Vec2D::new_world(-1.0, -1.0), Vec2D::new_world(1.0, 1.0)]);

        assert_eq!(buffer.len(), 3);

        buffer
            .iter()
            .for_each(|x| assert_eq!(x.color(), Some(Color::default())));
    }

    #[test]
    fn get_bounds() {
        let mut storage = Storage::new();

        storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(-5.0, -5.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
            ],
            Default::default(),
        )));
        storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(5.0, 5.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
                PathCommand::LineTo(Vec2D::new_world(1.0, 1.0)),
            ],
            Default::default(),
        )));

        assert_eq!(
            storage.get_bounds(),
            Some([Vec2D::new_world(-7.0, -7.0), Vec2D::new_world(7.0, 7.0)])
        );
    }

    #[test]
    fn draw_oder_is_respected() {
        let mut storage = Storage::new();

        storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(0.0, 2.0)),
            ],
            Style::red_line(),
        )));
        storage.add(Box::new(Path::from_parts(
            vec![
                PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
                PathCommand::LineTo(Vec2D::new_world(0.0, 1.0)),
            ],
            Default::default(),
        )));

        let commands =
            storage.draw_commands([Vec2D::new_world(-1.0, -1.0), Vec2D::new_world(1.0, 3.0)]);

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].color(), Some(Color::red()));
        assert_eq!(commands[1].color(), Some(Default::default()));
    }
}
