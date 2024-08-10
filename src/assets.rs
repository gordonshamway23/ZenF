use agb::{display::object::{Graphics, TagMap}, include_aseprite, include_background_gfx, include_wav};




include_background_gfx!(priv_menu_bg_gfx, bg => "assets/tex/bg.png", menu => "assets/tex/menu.png", help1 => "assets/tex/help_splitted1.png", help2 => "assets/tex/help_splitted2.png");
pub mod menu_bg_gfx {
    pub use super::priv_menu_bg_gfx::*;
}

include_background_gfx!(priv_game_bg_gfx, bg => "assets/tex/bg.png", tiles => "assets/tex/tiles.png", nums => "assets/tex/nums.png", menu => "assets/tex/menu.png");
pub mod game_bg_gfx {
    pub use super::priv_game_bg_gfx::*;
}


pub static GRAPHICS: &Graphics = include_aseprite!(
    "assets/tex/help_arrows.aseprite",
    "assets/tex/hover.aseprite",
    "assets/tex/arrows.aseprite"
);
//pub static SPRITES: &[Sprite] = GRAPHICS.sprites();
pub static TAG_MAP: &TagMap = GRAPHICS.tags();


pub static CURSOR_MOVE_SOUND: &[u8] = include_wav!("assets/snd/hover.wav");
pub static SELECT_SOUND: &[u8] = include_wav!("assets/snd/select.wav");
pub static FLATTEN_DEFLATTEN_SOUND: &[u8] = include_wav!("assets/snd/flatten_anim.wav");
pub static SOLVED_SOUND: &[u8] = include_wav!("assets/snd/solved.wav");