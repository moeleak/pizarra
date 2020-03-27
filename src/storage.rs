use std::collections::HashMap;

use rstar::RTree;

use super::shape::{Shape, ShapeId};
use super::draw_commands::DrawCommand;

pub struct ShapeStorage {
    shape_count: usize,
    layers: HashMap<i32, RTree<Shape>>,
}

/// A storage struct that organizes shapes by their zoom level and allos for
/// fast queries given a zoom and a bbox.
impl ShapeStorage {
    pub fn new() -> ShapeStorage {
        ShapeStorage {
            shape_count: 0,
            layers: HashMap::new(),
        }
    }

    pub fn iter(&self) -> ShapeIterator {
        unimplemented!()
    }

    pub fn add(&mut self, shape: Shape, zoom: i32) -> ShapeId {
        let subtree = self.layers.entry(zoom).or_insert(RTree::new());

        subtree.insert(shape);

        self.shape_count += 1;

        ShapeId::from(self.shape_count)
    }

    pub fn remove(&mut self, id: ShapeId) -> Shape {
        unimplemented!()
    }

    pub fn last_mut(&mut self) -> Option<&mut Shape> {
        unimplemented!()
    }

    pub fn pop(&mut self) -> Option<Shape> {
        unimplemented!()
    }

    pub fn shape_count(&self) -> usize {
        self.shape_count
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn draw(&self, layer: i32, bbox: [f64; 4]) -> impl Iterator<Item=DrawCommand> {
        Vec::new().into_iter()
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
    use super::ShapeStorage;
    use crate::shape::{Shape, ShapeId, Line, Rectangle, Circle};
    use crate::color::Color;

    #[test]
    fn test_add_shapes_at_zoom() {
        let mut storage = ShapeStorage::new();
        let shapes: Vec<(Shape, i32)> = vec![
            (Line::new(Color::red()), 0),
            (Rectangle::new(Color::green()), 1),
            (Circle::new(Color::blue()), -1),
        ];
        let ids: Vec<_> = shapes.into_iter().map(|(shape, zoom)| {
            storage.add(shape, zoom)
        }).collect();

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

        storage.add(Line::new(Color::blue()), 0);

        let last_shape = storage.last_mut().unwrap();

        last_shape.handle([0.0, 0.0]);
        last_shape.handle([1.0, 0.0]);
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

        let id = storage.add(Line::new(Color::green()), 0);

        assert_eq!(storage.shape_count(), 1);

        let data = storage.remove(id);

        assert_eq!(storage.shape_count(), 0);

        assert!(false, "given a shape identifier remove it and return enough info to add it again");
        assert!(false, "this is to implement ctrl-z after adding a shape");
    }

    #[test]
    fn test_iter_by_zoom_and_bounds() {
        assert!(false, "return an iterator over all the shapes that are visible given a zoom level and a bbox");
        assert!(false, "consider returning an iterator over draw commands instead of shapes");
        assert!(false, "each command must include a relative zoom level and a relative location");
        assert!(false, "shapes at different layers are returned by their relative_layer");
    }

    #[test]
    fn test_relative_coordinates() {
        assert!(false, "at each zoom level coordinates should be relative, such that then rendering from it no transformation will be needed");
    }
}
