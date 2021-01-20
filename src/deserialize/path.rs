use crate::shape::path::PathCommand;

use super::Error;

pub fn parse_path_commands(s: &str) -> Result<Vec<PathCommand>, Error> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::parse_path_commands;

    #[test]
    fn test_parse_path_commands() {
        assert!(false, "Draw a line in pizarra, save it and try to parse the result");
        assert!(false, "Do the same with a polygon and a rectangle");
    }
}
