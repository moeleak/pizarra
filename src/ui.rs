#[derive(Debug)]
pub struct Flags {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Default for Flags {
    fn default() -> Flags {
        Flags {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
}
