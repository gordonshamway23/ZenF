use agb::display::object::{ OamManaged, Object};
use agb::display::tiled::{MapLoan, RegularMap};
use agb::fixnum::Vector2D;
use agb::input::Button;
use agb::rng::RandomNumberGenerator;

use agb::sound::mixer::{Mixer, SoundChannel};

use agb::display::{
        tiled::{Tiled0, VRamManager, RegularBackgroundSize, TiledMap},
        Priority,
    };

use crate::assets::*;
use crate::game;
use crate::game::logic::{MAX_PLAYING_FIELD_HEIGHT, MAX_PLAYING_FIELD_WIDTH};
use crate::util::gbaex::ButtonControllerAutoRepeat;






const COUNT_HELP_PAGES : i32 = 5;

#[derive(PartialEq, Clone, Copy)]
pub enum MenuExitMode {
    DoNotExit,
    Exit_StartNewGame,
    Exit_ContinueGame
}


#[derive(PartialEq, Clone, Copy)]
enum SelectMenuItem {
    ContinueGame,
    StartNewGame,
    BoardSizeX,
    BoardSizeY,
    Sound,
    HowToPlay,
}

pub struct MenuView<'gba> {

    selected_menu_item : SelectMenuItem,

    gba_bg_bg : MapLoan<'gba, RegularMap>,
    gba_bg_bg_update_req : bool,
    gba_bg_menu : MapLoan<'gba, RegularMap>,
    gba_bg_help1 : MapLoan<'gba, RegularMap>,
    gba_bg_help2 : MapLoan<'gba, RegularMap>,
    gba_bg_menu_update_req : bool,

    gba_obj_anim_counter : usize,
    gba_obj_anim_frame : usize,
    gba_obj_arrows : [Object<'gba>; 2],
    gba_objs_update_req : bool,

    help_shown : bool,
    help_page_nr : i32,

    exit_mode : MenuExitMode,

}

impl<'gba> MenuView<'gba> {
 
    pub fn new(gba_tiled0 : &'gba Tiled0, gba_vram : &mut VRamManager, gba_oam : &'gba OamManaged, first_start : bool) -> Self {
        
        gba_vram.set_background_palettes(menu_bg_gfx::PALETTES);

        MenuView {

            selected_menu_item : if first_start {SelectMenuItem::StartNewGame} else {SelectMenuItem::ContinueGame},

            gba_bg_bg : gba_tiled0.background(
                Priority::P3,
                RegularBackgroundSize::Background32x32,
                menu_bg_gfx::bg.tiles.format(),
            ),
            gba_bg_bg_update_req : true,

            gba_bg_menu : gba_tiled0.background(
                Priority::P2,
                RegularBackgroundSize::Background32x32,
                menu_bg_gfx::menu.tiles.format(),
            ),
            gba_bg_help1 : gba_tiled0.background(
                Priority::P1,
                RegularBackgroundSize::Background32x32,
                menu_bg_gfx::help1.tiles.format(),
            ),
            gba_bg_help2 : gba_tiled0.background(
                Priority::P0,
                RegularBackgroundSize::Background32x32,
                menu_bg_gfx::help2.tiles.format(),
            ),
            gba_bg_menu_update_req : true,

            gba_obj_anim_counter : 0,
            gba_obj_anim_frame : 0,
            gba_obj_arrows : [
                gba_oam.object_sprite(TAG_MAP.get("HelpArrowUp").sprite(0)),
                gba_oam.object_sprite(TAG_MAP.get("HelpArrowDown").sprite(0)) ],
            gba_objs_update_req : true,

            help_shown : false,
            help_page_nr : 0,

            exit_mode : MenuExitMode::DoNotExit,

        }
    }

    pub fn get_exit_mode(&self) -> MenuExitMode {
        self.exit_mode
    }

    fn update_gba_bgs(&mut self, gba_vram : &mut VRamManager, game_settings : &mut game::Settings) {

        if self.gba_bg_bg_update_req {
            self.gba_bg_bg_update_req = false;

            if self.exit_mode!=MenuExitMode::DoNotExit {
                //self.gba_bg_bg.set_visible(false);
                //self.gba_bg_bg.commit(gba_vram);

            } else {

                let tileset_bg = &menu_bg_gfx::bg.tiles;
                let tile_settings_bg = menu_bg_gfx::bg.tile_settings;

                let mut i_bg;
                let mut pos;

                let mut rng = RandomNumberGenerator::new();
                for y in 0..20 {
                    for x in 0..30 {
                        pos = (x as u16, y as u16);

                        if rng.gen().abs()%10==0 {
                            i_bg = 1 + (rng.gen().abs()%3) as usize;
                        } else {
                            i_bg = 0;
                        }
                        
                        self.gba_bg_bg.set_tile(gba_vram, pos, tileset_bg, tile_settings_bg[i_bg]);                
                    }
                }

                self.gba_bg_bg.set_visible(true);
                self.gba_bg_bg.commit(gba_vram);
            }
        }

        if self.gba_bg_menu_update_req {
            self.gba_bg_menu_update_req = false;
            if self.exit_mode!=MenuExitMode::DoNotExit {
                self.gba_bg_menu.set_visible(false);
                self.gba_bg_menu.commit(gba_vram);

                self.gba_bg_help1.set_visible(false);
                self.gba_bg_help2.set_visible(false);
                self.gba_bg_help1.commit(gba_vram);
                self.gba_bg_help2.commit(gba_vram);

            } else {

                let tileset_menu = &menu_bg_gfx::menu.tiles;
                let tile_settings_menu = menu_bg_gfx::menu.tile_settings;

                let tileset_help1 = &menu_bg_gfx::help1.tiles;
                let tile_settings_help1 = menu_bg_gfx::help1.tile_settings;
                let tileset_help2 = &menu_bg_gfx::help2.tiles;
                let tile_settings_help2 = menu_bg_gfx::help2.tile_settings;

                let mut i_menu;
                let mut i_help1;
                let mut i_help2;
                let mut pos;

                for y in 0..20 {
                    for x in 0..30 {
                        pos = (x as u16, y as u16);
                        i_menu=0;
                        i_help1=29;
                        i_help2=29;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);   
                        self.gba_bg_help1.set_tile(gba_vram, pos, tileset_help1, tile_settings_help1[i_help1]);
                        self.gba_bg_help2.set_tile(gba_vram, pos, tileset_help2, tile_settings_help2[i_help2]);                
                    }
                }


                if self.help_shown {

                    //help menu 

                    //text
                    let ty_per_page = [0, 12, 23, 37, 51];
                    let th_per_page = [11, 10, 13, 13, 8 ];
                    for y in 0..th_per_page[self.help_page_nr as usize] {
                        for x in 0..30 {
                            pos = (x as u16, (y+1) as u16);
                            let ty = ty_per_page[self.help_page_nr as usize] + y;
                            if ty>=34 {
                                i_help2 = (ty-34)*30 + x ;
                                self.gba_bg_help2.set_tile(gba_vram, pos, tileset_help2, tile_settings_help2[i_help2]);  
                            } else {
                                i_help1 = ty*30 + x ;
                                self.gba_bg_help1.set_tile(gba_vram, pos, tileset_help1, tile_settings_help1[i_help1]);              
                            }
                        }
                    }

                    //page nr
                    {
                        pos = (13, 18);
                        i_menu = 128+self.help_page_nr as usize +1;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (14, 18);
                        i_menu = 128+31;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (15, 18);
                        i_menu = 128+COUNT_HELP_PAGES as usize;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }

                } else {
                    //main menu

                    //title

                    for y in 0..4 {
                        for x in 0..10 {
                            pos = (10 + x as u16, 1 + y as u16);
                            i_menu = y*32 + x + 22;
                            self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                        }
                    }


                    let l = 11;

                    let mut y = 6;
                    // continue game
                    {
                        pos = (l - 2 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::ContinueGame {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..8 {
                        pos = (l + x as u16, y as u16);
                        i_menu = (if game_settings.playing_field_data.is_none() {64} else {96}) + 9 + x;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                    }
                    y+=2;

                    // start new game
                    {
                        pos = (l - 2 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::StartNewGame {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..8 {
                        pos = (l + x as u16, y as u16);
                        i_menu = 96+x;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                    }
                    y+=2;

                    // board size
                    {
                        pos = (l - 2 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::BoardSizeX || self.selected_menu_item==SelectMenuItem::BoardSizeY {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..6 {
                        pos = (l + x as u16, y as u16);
                        i_menu = 32+x;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                    }
                    // x
                    {
                        pos = (l + 6 as u16, y as u16);
                        i_menu = 32+6;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 7 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::BoardSizeX {1} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 8 as u16, y as u16);
                        i_menu = 128+game_settings.playing_field_width;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 9 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::BoardSizeX {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    y+=1;
                    // y
                    {
                        pos = (l + 6 as u16, y as u16);
                        i_menu = 32+7;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 7 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::BoardSizeY {1} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 8 as u16, y as u16);
                        i_menu = 128+game_settings.playing_field_height;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 9 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::BoardSizeY {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    y+=2;
                    



                    // sound
                    {
                        pos = (l - 2 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::Sound {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..3 {
                        pos = (l + x as u16, y as u16);
                        i_menu = 2*32+17+x;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                    }
                    {
                        pos = (l + 3 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::Sound {1} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..2 {
                        pos = (l + 4 + x as u16, y as u16);
                        i_menu = 2*32+20+x + (if game_settings.sound {0} else {32});
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    {
                        pos = (l + 6 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::Sound {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    y+=2;

                    // how to play
                    {
                        pos = (l - 2 as u16, y as u16);
                        i_menu = if self.selected_menu_item==SelectMenuItem::HowToPlay {2} else {0};
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]); 
                    }
                    for x in 0..7 {
                        pos = (l + x as u16, y as u16);
                        i_menu = 64+x;
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                    }
                    y+=2;


                    // credits
                    for y in 0..2 {
                        for x in 0..14 {
                            pos = ((30-14+x) as u16, (20-2+y) as u16);
                            i_menu = y*32 + 8+x;
                            self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);              
                        }
                    }

                }

                self.gba_bg_menu.set_visible(true);
                self.gba_bg_menu.commit(gba_vram);

                self.gba_bg_help1.set_visible(true);
                self.gba_bg_help2.set_visible(true);
                self.gba_bg_help1.commit(gba_vram);
                self.gba_bg_help2.commit(gba_vram);

            }
        }


    }


    fn update_gba_objs(&mut self, gba_oam : &OamManaged) {

        if self.gba_objs_update_req {
            self.gba_objs_update_req = false;

            if !self.help_shown || self.exit_mode!=MenuExitMode::DoNotExit {
                for i in 0..2usize {
                    self.gba_obj_arrows[i].hide();
                }
                gba_oam.commit();
                return;
            }
            

            self.gba_obj_arrows[0].set_sprite(gba_oam.sprite(TAG_MAP.get("HelpArrowUp").sprite(self.gba_obj_anim_frame)));
            self.gba_obj_arrows[1].set_sprite(gba_oam.sprite(TAG_MAP.get("HelpArrowDown").sprite(self.gba_obj_anim_frame)));

            if self.help_page_nr<=0 {
                self.gba_obj_arrows[0].hide();
            } else {                
                self.gba_obj_arrows[0].set_position(Vector2D::new(29*8, 0)).show();
            } 

            if self.help_page_nr>=COUNT_HELP_PAGES-1 {
                self.gba_obj_arrows[1].hide();
            } else {                
                self.gba_obj_arrows[1].set_position(Vector2D::new(29*8, 19*8)).show();
            } 


            gba_oam.commit();
        }
    }

    pub fn update(&mut self, gba_vram : &mut VRamManager, gba_oam : &OamManaged, game_settings : &mut game::Settings) {

        self.gba_obj_anim_counter+=1;
        if self.gba_obj_anim_counter>=10 {
            self.gba_obj_anim_counter=0;
            self.gba_obj_anim_frame = (self.gba_obj_anim_frame+1)%2;
            self.gba_objs_update_req=true;
        }

        self.update_gba_bgs(gba_vram, game_settings);
        self.update_gba_objs(gba_oam);

    }

    pub fn handle_input(&mut self, gba_input : &ButtonControllerAutoRepeat, gba_mixer : &mut Mixer, game_settings : &mut game::Settings) {

        if self.help_shown {

            if gba_input.btn_ctrl.is_just_pressed(Button::A.union(Button::B).union(Button::START).union(Button::SELECT)) {
                self.help_shown=false;
                self.gba_bg_menu_update_req=true;
                self.gba_objs_update_req=true;

                if game_settings.sound {
                    let mut sc = SoundChannel::new(&SELECT_SOUND);
                    sc.stereo();
                    gba_mixer.play_sound(sc);
                }
            }
            else if gba_input.btn_ctrl.is_just_pressed(Button::UP) {
                if self.help_page_nr>0 {
                    self.help_page_nr -= 1;
                    self.gba_bg_menu_update_req=true;
                    self.gba_objs_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
            else if gba_input.btn_ctrl.is_just_pressed(Button::DOWN) {
                if self.help_page_nr<COUNT_HELP_PAGES-1 {
                    self.help_page_nr += 1;
                    self.gba_bg_menu_update_req=true;
                    self.gba_objs_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
        } else {
            
            if gba_input.is_just_pressed_or_auto_repeated(Button::UP) {
                if self.selected_menu_item!=SelectMenuItem::ContinueGame {
                    self.selected_menu_item = match self.selected_menu_item {
                        SelectMenuItem::ContinueGame => SelectMenuItem::ContinueGame,
                        SelectMenuItem::StartNewGame => SelectMenuItem::ContinueGame,
                        SelectMenuItem::BoardSizeX => SelectMenuItem::StartNewGame,
                        SelectMenuItem::BoardSizeY => SelectMenuItem::BoardSizeX,
                        SelectMenuItem::Sound => SelectMenuItem::BoardSizeY,
                        SelectMenuItem::HowToPlay => SelectMenuItem::Sound,
                    };
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
            else if gba_input.is_just_pressed_or_auto_repeated(Button::DOWN) {
                if self.selected_menu_item!=SelectMenuItem::HowToPlay {
                    self.selected_menu_item = match self.selected_menu_item {
                        SelectMenuItem::ContinueGame => SelectMenuItem::StartNewGame,
                        SelectMenuItem::StartNewGame => SelectMenuItem::BoardSizeX,
                        SelectMenuItem::BoardSizeX => SelectMenuItem::BoardSizeY,
                        SelectMenuItem::BoardSizeY => SelectMenuItem::Sound,
                        SelectMenuItem::Sound => SelectMenuItem::HowToPlay,
                        SelectMenuItem::HowToPlay => SelectMenuItem::HowToPlay,
                    };
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
            else if gba_input.is_just_pressed_or_auto_repeated(Button::LEFT) {
                if self.selected_menu_item == SelectMenuItem::BoardSizeX && game_settings.playing_field_width>5 {
                    game_settings.playing_field_width -=1;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                } else if self.selected_menu_item == SelectMenuItem::BoardSizeY && game_settings.playing_field_height>5 {
                    game_settings.playing_field_height -=1;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                } else if self.selected_menu_item == SelectMenuItem::Sound {
                    game_settings.sound=!game_settings.sound;
                    self.gba_bg_menu_update_req=true;
                    
                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
            else if gba_input.is_just_pressed_or_auto_repeated(Button::RIGHT) {
                if self.selected_menu_item == SelectMenuItem::BoardSizeX && game_settings.playing_field_width<MAX_PLAYING_FIELD_WIDTH {
                    game_settings.playing_field_width +=1;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                } else if self.selected_menu_item == SelectMenuItem::BoardSizeY && game_settings.playing_field_height<MAX_PLAYING_FIELD_HEIGHT{
                    game_settings.playing_field_height +=1;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                } else if self.selected_menu_item == SelectMenuItem::Sound {
                    game_settings.sound=!game_settings.sound;
                    self.gba_bg_menu_update_req=true;
                    
                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&CURSOR_MOVE_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
            
            
            if gba_input.btn_ctrl.is_just_pressed(Button::A.union(Button::B)) {
                if self.selected_menu_item == SelectMenuItem::StartNewGame {
                    self.exit_mode = MenuExitMode::Exit_StartNewGame;
                    self.gba_bg_bg_update_req=true;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
                else if self.selected_menu_item == SelectMenuItem::ContinueGame && game_settings.playing_field_data.is_some() {
                    self.exit_mode = MenuExitMode::Exit_ContinueGame;
                    self.gba_bg_bg_update_req=true;
                    self.gba_bg_menu_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
                else if self.selected_menu_item == SelectMenuItem::HowToPlay {
                    self.help_shown=true;
                    self.gba_bg_menu_update_req=true;
                    self.gba_objs_update_req=true;

                    if game_settings.sound {
                        let mut sc = SoundChannel::new(&SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }

        }
    }



}
