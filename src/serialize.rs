use super::app::App;
use super::storage::ShapeStorage;
use super::shape::ShapeId;
use super::color::Color;

pub trait Serialize {
    fn serialize(&self) -> String;
}

pub trait ShapeSerialize {
    fn serialize(&self, id: ShapeId, color: Color) -> String;
}

impl Serialize for App {
    fn serialize(&self) -> String {
        let layer_string = self.storage().serialize();
        let dimensions = self.get_dimentions();

        format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>
<!-- Created with Pizarra (https://github.com/categulario/pizarra) -->

<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">
<svg
  xmlns:cc=\"http://creativecommons.org/ns#\"
  xmlns:svg=\"http://www.w3.org/2000/svg\"
  xmlns=\"http://www.w3.org/2000/svg\"
  width=\"{width}\"
  height=\"{height}\"
  viewBox=\"0 0 {width} {height}\"
  version=\"1.1\"
  id=\"svg8\"
>
  {layers}
</svg>", width=dimensions[0], height=dimensions[1], layers=layer_string)
    }
}

impl Serialize for ShapeStorage {
    fn serialize(&self) -> String {
        let layers: Vec<String> = self.iter_layers().map(|(zoom, layer_tree)| {
            let shape_strings: Vec<String> = layer_tree.iter().map(|shape| {
                shape.serialize()
            }).collect();
            let strings = shape_strings.join("\n");

            format!("<g inkscape:label=\"Zoom {zoom}\" inkscape:groupmode=\"layer\" id=\"layer{zoom}\">{strings}</g>", zoom=zoom, strings=strings)
        }).collect();

        layers.join("\n")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn test_can_serialize() {
        assert!(false, "the storage can be serialized");
        assert!(false, "the size of the svg is the last window size");
        assert!(false, "shapes of different layers are in different layers that inkscape can use");
        assert!(false, "sizes and thicknesses are as expected");
    }
}
