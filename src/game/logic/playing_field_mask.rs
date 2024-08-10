use super::*;

pub struct PlayingFieldMask {
    pub width: usize,
    pub height: usize,
    pub mask: [bool; MAX_PLAYING_FIELD_AREA]
}

impl PlayingFieldMask {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && width <= MAX_PLAYING_FIELD_WIDTH);
        assert!(height > 0 && height <= MAX_PLAYING_FIELD_HEIGHT);
        PlayingFieldMask {
            width,
            height,
            mask: [true; MAX_PLAYING_FIELD_AREA]
        }
    }
}