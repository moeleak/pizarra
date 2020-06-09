use super::app::App;
use super::storage::ShapeStorage;
use super::shape::ShapeId;
use super::color::Color;
use super::zoom::Zoom;

pub trait Serialize {
    fn serialize(&self) -> String;
}

pub trait ShapeSerialize {
    fn serialize(&self, zoom: Zoom) -> String;
}

pub trait ShapeTraitSerialize {
    fn serialize(&self, id: ShapeId, color: Color, zoom: Zoom) -> String;
}

fn bounds_to_size(bounds: [[f64; 2]; 2]) -> (f64, f64) {
    (bounds[1][0] - bounds[0][0], bounds[1][1] - bounds[0][1])
}

impl Serialize for App {
    fn serialize(&self) -> String {
        String::new()
    }
}

impl Serialize for ShapeStorage {
    fn serialize(&self) -> String {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::bounds_to_size;

    #[test]
    #[ignore]
    fn test_can_serialize() {
        assert!(false, "the storage can be serialized");
        assert!(false, "the size of the svg is the last window size");
        assert!(false, "shapes of different layers are in different layers that inkscape can use");
        assert!(false, "sizes and thicknesses are as expected");
    }

    #[test]
    fn test_bounds_to_size() {
        let bounds = [[-5.0, -2.0], [3.0, 4.0]];

        assert_eq!(bounds_to_size(bounds), (8.0, 6.0));
    }
}
