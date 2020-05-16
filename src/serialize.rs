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

impl<G> Serialize for App<G> {
    fn serialize(&self) -> String {
        let layer_string = self.storage().serialize();
        let bounds = self.storage().get_bounds(Zoom::from(0)).unwrap();

        let (bboardwith, bboardheight) = bounds_to_size(bounds);
        let blackboard = format!("<rect
    style=\"fill:{color};fill-opacity:1;stroke:none;\"
    id=\"blackboard\"
    width=\"{bboardwith}\"
    height=\"{bboardheight}\"
    x=\"0\"
    y=\"0\" />", color=Color::Black, bboardwith=bboardwith+20.0, bboardheight=bboardheight+20.0);

        format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>
<!-- Created with Pizarra (https://github.com/categulario/pizarra) -->

<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">
<svg
  xmlns=\"http://www.w3.org/2000/svg\"
  width=\"{width}\"
  height=\"{height}\"
  viewBox=\"0 0 {width} {height}\"
  version=\"1.1\"
>
  {blackboard}
  {layers}
</svg>", width=bboardwith, height=bboardheight, layers=layer_string, blackboard=blackboard)
    }
}

impl Serialize for ShapeStorage {
    fn serialize(&self) -> String {
        let layers: Vec<String> = self.iter_layers().map(|(zoom, layer_tree)| {
            let shape_strings: Vec<String> = layer_tree.iter().map(|shape| {
                shape.serialize(*zoom)
            }).collect();
            let strings = shape_strings.join("\n");

            format!("<g id=\"layer{zoom}\">{strings}</g>", zoom=zoom, strings=strings)
        }).collect();

        layers.join("\n")
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
