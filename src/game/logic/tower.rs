use agb::fixnum::Rect;

pub struct Tower {
    pub x: usize,
    pub y: usize,
    pub height: i32,
    pub flattened_height: i32,
    pub bounds: Rect<i32>,
}
