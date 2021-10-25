#[derive(Debug, Copy, Clone)]
pub enum Key {
    Shift,
    Escape,

    /// Everything else
    Unknown,
}
