pub const MAX_PLAYING_FIELD_WIDTH: usize = 30; 
pub const MAX_PLAYING_FIELD_HEIGHT: usize = 20; //means max tower height 49, so max 2 digits
pub const MAX_PLAYING_FIELD_AREA: usize = MAX_PLAYING_FIELD_WIDTH * MAX_PLAYING_FIELD_HEIGHT;
pub const NO_TOWER: i32 = -1;

macro_rules! lin2d {
    ($x:expr,$y:expr,$width:expr) => {
        (($x) + ($y) * ($width))
    };
}
pub(crate) use lin2d; //required to make macro behave like variables (scopewise)
