use super::storage::ShapeStorage;

pub trait Serialize {
    fn serialize(&self) -> String;
}

impl Serialize for ShapeStorage {
    fn serialize(&self) -> String {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>
<!-- Created with Pizarra (https://github.com/categulario/pizarra) -->

<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">
<svg
  xmlns:cc=\"http://creativecommons.org/ns#\"
  xmlns:svg=\"http://www.w3.org/2000/svg\"
  xmlns=\"http://www.w3.org/2000/svg\"
  width=\"400px\"
  height=\"400px\"
  viewBox=\"0 0 400 400\"
  version=\"1.1\"
  id=\"svg8\"
>
  <g id=\"layer1\">
  </g>
</svg>")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_serialize() {
        assert!(false, "the storage can be serialized");
        assert!(false, "the size of the svg is the last window size");
        assert!(false, "shapes of different layers are in different layers that inkscape can use");
        assert!(false, "sizes and thicknesses are as expected");
    }
}
