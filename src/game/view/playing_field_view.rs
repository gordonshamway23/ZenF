extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;

use agb::display::object::{Graphics, OamManaged, Object, Sprite, SpriteVram, TagMap};
use agb::display::tiled::{MapLoan, RegularMap};
use agb::fixnum::Vector2D;
use agb::input::Button;
use agb::rng::RandomNumberGenerator;

use agb::sound::mixer::{Mixer, SoundChannel};
use agb::{include_background_gfx, include_aseprite, include_wav};

use agb::display::{
        tiled::{Tiled0, VRamManager, RegularBackgroundSize, TiledMap},
        Priority,
    };

use crate::game::Settings;
use crate::util::gbaex::ButtonControllerAutoRepeat;
use crate::assets::*;

use super::super::logic::*;

use super::*;



#[derive(PartialEq, Clone, Copy)]
pub enum IngameExitMode {
    DoNotExit,
    Exit_BoardCompleted,
    Exit_BoardNotCompleted
}


#[derive(PartialEq)]
enum PlayingFieldViewInputMode {
    MoveSelect,
    Flatten,
    Deflatten,
}

pub struct PlayingFieldView<'gba> {
    hovered_tile_xy : (i32, i32),
    //hovered_tower_xy : Option<(i32, i32)>,
    selected_tower_xy : Option<(i32, i32)>,

    input_mode : PlayingFieldViewInputMode,

    tower_colors_mapping : [usize; TOWER_COLORS_COUNT],
    tower_num_light_dark_mapping : [usize; TOWER_COLORS_COUNT],

    gba_bg_bg : MapLoan<'gba, RegularMap>,
    gba_bg_bg_update_req : bool,
    gba_bg_tiles : MapLoan<'gba, RegularMap>,
    gba_bg_nums : MapLoan<'gba, RegularMap>,
    gba_bg_tiles_and_nums_update_req : bool,
    gba_bg_menu : MapLoan<'gba, RegularMap>,
    gba_bg_menu_update_req : bool,

    gba_obj_hover : Object<'gba>,
    gba_obj_hover_anim_counter : usize,
    gba_obj_hover_anim_frame : usize,
    gba_obj_arrows : [Object<'gba>; 4],
    gba_objs_update_req : bool,

    exit_mode : IngameExitMode,
}

impl<'gba> PlayingFieldView<'gba> {
 
    pub fn new(gba_tiled0 : &'gba Tiled0, gba_vram : &mut VRamManager, gba_oam : &'gba OamManaged) -> Self {
        

        gba_vram.set_background_palettes(game_bg_gfx::PALETTES);


        let mut pfv = PlayingFieldView {

            hovered_tile_xy: (0,0),
            //hovered_tower_xy: None,
            selected_tower_xy: None,

            input_mode: PlayingFieldViewInputMode::MoveSelect,

            tower_colors_mapping: [0; TOWER_COLORS_COUNT],
            tower_num_light_dark_mapping: [0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0],

            gba_bg_bg : gba_tiled0.background(
                Priority::P3,
                RegularBackgroundSize::Background32x32,
                game_bg_gfx::bg.tiles.format(),
            ),
            gba_bg_bg_update_req : true,
            gba_bg_tiles : gba_tiled0.background(
                Priority::P2,
                RegularBackgroundSize::Background32x32,
                game_bg_gfx::tiles.tiles.format(),
            ),
            gba_bg_nums : gba_tiled0.background(
                Priority::P1,
                RegularBackgroundSize::Background32x32,
                game_bg_gfx::nums.tiles.format(),
            ),
            gba_bg_tiles_and_nums_update_req : true,
            gba_bg_menu : gba_tiled0.background(
                Priority::P0,
                RegularBackgroundSize::Background32x32,
                game_bg_gfx::menu.tiles.format(),
            ),
            gba_bg_menu_update_req : true,

            gba_obj_hover : gba_oam.object_sprite(TAG_MAP.get("Hover").sprite(0)),
            gba_obj_hover_anim_counter : 0,
            gba_obj_hover_anim_frame : 0,
            gba_obj_arrows : [
                gba_oam.object_sprite(TAG_MAP.get("ArrowLeft").sprite(0)),
                gba_oam.object_sprite(TAG_MAP.get("ArrowRight").sprite(0)),
                gba_oam.object_sprite(TAG_MAP.get("ArrowUp").sprite(0)),
                gba_oam.object_sprite(TAG_MAP.get("ArrowDown").sprite(0)) ],
            gba_objs_update_req : true,

            exit_mode : IngameExitMode::DoNotExit,
        };

        for i in 0..TOWER_COLORS_COUNT {
            pfv.tower_colors_mapping[i]=i;
        }

        pfv
    }

    pub fn save_as_u8_vec(&self, pf: &PlayingField) -> Vec<u8> {
        let mut data = pf.save_as_u8_vec();

        data.push(self.hovered_tile_xy.0 as u8);
        data.push(self.hovered_tile_xy.1 as u8);
        data.push(if self.selected_tower_xy.is_some() {1} else {0});
        if let Some(sel_t) = self.selected_tower_xy {
            data.push(sel_t.0 as u8);
            data.push(sel_t.1 as u8);
        }
        data.push(match self.input_mode {
            PlayingFieldViewInputMode::MoveSelect => 0,
            PlayingFieldViewInputMode::Flatten => 1,
            PlayingFieldViewInputMode::Deflatten => 2
        });
        for i in 0..TOWER_COLORS_COUNT {
            data.push(self.tower_colors_mapping[i] as u8);
        }

        data
    }

    pub fn load_from_u8_vec(&mut self, pf: &mut PlayingField, data : &Vec<u8>) -> usize {
        let mut di : usize = pf.load_from_u8_vec(data);

        self.hovered_tile_xy.0 = data[di] as i32; di+=1;
        self.hovered_tile_xy.1 = data[di] as i32; di+=1;
        if data[di]!=0 {
            self.selected_tower_xy = Some((data[di+1] as i32, data[di+2] as i32));
            di+=3;
        } else {
            self.selected_tower_xy = None;
            di+=1;
        }
        self.input_mode = match data[di] {
            0 => PlayingFieldViewInputMode::MoveSelect,
            1 => PlayingFieldViewInputMode::Flatten,
            2 => PlayingFieldViewInputMode::Deflatten,
            _ => PlayingFieldViewInputMode::MoveSelect
        };
        di+=1;
        for i in 0..TOWER_COLORS_COUNT {
            self.tower_colors_mapping[i] = data[di+i] as usize;
        }
        di+=TOWER_COLORS_COUNT;

        di
    }

    pub fn get_exit_mode(&self) -> IngameExitMode {
        self.exit_mode
    }

    pub fn reset_input(&mut self) {
        self.hovered_tile_xy = (0,0);
        //self.hovered_tower_xy = None;
        self.selected_tower_xy = None;
    }

    pub fn reset_to_start_state(&mut self, pf: &mut PlayingField) {
        self.reset_input();
        pf.reset_to_start_state();
    }

    pub fn set_to_solution_state(&mut self, pf: &mut PlayingField) {
        self.reset_input();
        pf.set_to_solution_state();
    }

    pub fn init_with_random_towers(&mut self, pf: &mut PlayingField, seed : Option<[u32; 4]>) {
        self.reset_input();

        let mut rng = if seed.is_some() {RandomNumberGenerator::new_with_seed(seed.unwrap())} else {RandomNumberGenerator::new()};
        pf.init_with_random_towers(&mut rng);

        crate::util::rng::fisher_yates_shuffle_arr_inplace(&mut self.tower_colors_mapping, &mut rng);
    }

    fn update_gba_bgs(&mut self, pf: &PlayingField, gba_vram : &mut VRamManager) {

        if self.gba_bg_bg_update_req {
            self.gba_bg_bg_update_req = false;

            if self.exit_mode!=IngameExitMode::DoNotExit {
                //self.gba_bg_bg.set_visible(false);
                //self.gba_bg_bg.commit(gba_vram);

            } else {

                let tileset_bg = &game_bg_gfx::bg.tiles;
                let tile_settings_bg = game_bg_gfx::bg.tile_settings;

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

        if self.gba_bg_tiles_and_nums_update_req {
            self.gba_bg_tiles_and_nums_update_req = false;


            if self.exit_mode!=IngameExitMode::DoNotExit {
                self.gba_bg_tiles.set_visible(false);
                self.gba_bg_tiles.commit(gba_vram);

                self.gba_bg_nums.set_visible(false);
                self.gba_bg_nums.commit(gba_vram);
                
            } else {

                let tileset_tiles = &game_bg_gfx::tiles.tiles;
                let tile_settings_tiles = game_bg_gfx::tiles.tile_settings;

                let tileset_nums = &game_bg_gfx::nums.tiles;
                let tile_settings_nums = game_bg_gfx::nums.tile_settings;

                let mut i_tiles;
                let mut i_nums;
                let mut tower_idx : i32;
                let mut pos;
                let mut col;
                for y in 0..pf.height {
                    for x in 0..pf.width {
                        tower_idx = pf.field[lin2d!(x,y,pf.width)];
                        pos = (x as u16, y as u16);

                        if tower_idx!=NO_TOWER {
                            col = self.tower_colors_mapping[(tower_idx%32) as usize];
                            i_tiles = 2 + col;
                            let tower = &pf.towers[tower_idx as usize];
                            if tower.x==x && tower.y==y {
                                i_nums = 1 + tower.flattened_height as usize + self.tower_num_light_dark_mapping[col]*51;
                            } else {
                                i_nums = 0;
                            }
                        } else {
                            i_tiles = 1;
                            i_nums = 0;
                        }

                        self.gba_bg_tiles.set_tile(gba_vram, pos, tileset_tiles, tile_settings_tiles[i_tiles]);  
                        self.gba_bg_nums.set_tile(gba_vram, pos, tileset_nums, tile_settings_nums[i_nums]);                  
                    }
                }

                
                self.gba_bg_tiles.set_visible(true);
                self.gba_bg_tiles.set_scroll_pos(Vector2D::new(-120 + pf.width as i16 * 4, -80 + pf.height as i16 * 4));//center on screen
                self.gba_bg_tiles.commit(gba_vram);

                self.gba_bg_nums.set_visible(true);
                self.gba_bg_nums.set_scroll_pos(Vector2D::new(-120 + pf.width as i16 * 4, -80 + pf.height as i16 * 4));//center on screen
                self.gba_bg_nums.commit(gba_vram);

            }
        }


        if self.gba_bg_menu_update_req {
            self.gba_bg_menu_update_req = false;

            if self.exit_mode!=IngameExitMode::DoNotExit {
                self.gba_bg_menu.set_visible(false);
                self.gba_bg_menu.commit(gba_vram);

            } else {

                let tileset_menu = &game_bg_gfx::menu.tiles;
                let tile_settings_menu = game_bg_gfx::menu.tile_settings;

                let mut i_menu;
                let mut pos;

                for y in 0..20 {
                    for x in 0..30 {
                        pos = (x as u16, y as u16);

                        i_menu = 0;
                        
                        if pf.is_solved() {
                            if self.hovered_tile_xy.1>pf.height as i32/2 && x>=2 && x<=27 && y>=0 && y<4 {
                                i_menu = 32*5 + (y-0)*32 + (x-2);
                            }
                            else if self.hovered_tile_xy.1<=pf.height as i32/2 && x>=2 && x<=27 && y>=16 && y<20 {
                                i_menu = 32*5 + (y-16)*32 + (x-2);
                            }
                        }
                        
                        self.gba_bg_menu.set_tile(gba_vram, pos, tileset_menu, tile_settings_menu[i_menu]);                
                    }
                }

                self.gba_bg_menu.set_visible(true);
                self.gba_bg_menu.commit(gba_vram);
            }
        }

    }

    fn update_gba_objs(&mut self, pf: &PlayingField, gba_oam : &OamManaged) {


        if self.gba_objs_update_req {
            self.gba_objs_update_req = false;

            if self.exit_mode!=IngameExitMode::DoNotExit {
                self.gba_obj_hover.hide();
                for i in 0..4usize {
                    self.gba_obj_arrows[i].hide();
                }
                gba_oam.commit();
                return;
            }
            

            self.gba_obj_hover.set_sprite(gba_oam.sprite(TAG_MAP.get("Hover").sprite(self.gba_obj_hover_anim_frame)));

            let off = Vector2D::new(120 - pf.width as i32 * 4, 80 - pf.height as i32 * 4);
            self.gba_obj_hover.set_position(Vector2D::new(off.x+self.hovered_tile_xy.0*8 - 4, off.y+self.hovered_tile_xy.1*8 - 4)).show();


            self.gba_obj_arrows[0].set_sprite(gba_oam.sprite(TAG_MAP.get("ArrowLeft").sprite(self.gba_obj_hover_anim_frame)));
            self.gba_obj_arrows[1].set_sprite(gba_oam.sprite(TAG_MAP.get("ArrowRight").sprite(self.gba_obj_hover_anim_frame)));
            self.gba_obj_arrows[2].set_sprite(gba_oam.sprite(TAG_MAP.get("ArrowUp").sprite(self.gba_obj_hover_anim_frame)));
            self.gba_obj_arrows[3].set_sprite(gba_oam.sprite(TAG_MAP.get("ArrowDown").sprite(self.gba_obj_hover_anim_frame)));

            if self.input_mode == PlayingFieldViewInputMode::MoveSelect {
                self.gba_obj_arrows[0].hide();
                self.gba_obj_arrows[1].hide();
                self.gba_obj_arrows[2].hide();
                self.gba_obj_arrows[3].hide();
            } else if self.input_mode == PlayingFieldViewInputMode::Flatten {
                let selected_tower_idx = pf.field[lin2d!(self.hovered_tile_xy.0 as usize, self.hovered_tile_xy.1 as usize, pf.width)] as usize;
                let selected_tower = &pf.towers[selected_tower_idx];
                
                self.gba_obj_arrows[0].set_position(Vector2D::new(off.x+selected_tower.bounds.position.x*8 - 8, off.y+self.hovered_tile_xy.1*8)).show();
                self.gba_obj_arrows[1].set_position(Vector2D::new(off.x+(selected_tower.bounds.position.x+selected_tower.bounds.size.x)*8, off.y+self.hovered_tile_xy.1*8)).show();
                self.gba_obj_arrows[2].set_position(Vector2D::new(off.x+self.hovered_tile_xy.0*8, off.y+selected_tower.bounds.position.y*8 - 8)).show();
                self.gba_obj_arrows[3].set_position(Vector2D::new(off.x+self.hovered_tile_xy.0*8, off.y+(selected_tower.bounds.position.y+selected_tower.bounds.size.y)*8)).show();
            } else if self.input_mode == PlayingFieldViewInputMode::Deflatten {
                let selected_tower_idx = pf.field[lin2d!(self.hovered_tile_xy.0 as usize, self.hovered_tile_xy.1 as usize, pf.width)] as usize;
                let selected_tower = &pf.towers[selected_tower_idx];

                self.gba_obj_arrows[1].set_position(Vector2D::new(off.x+selected_tower.bounds.position.x*8 - 8, off.y+self.hovered_tile_xy.1*8)).show();
                self.gba_obj_arrows[0].set_position(Vector2D::new(off.x+(selected_tower.bounds.position.x+selected_tower.bounds.size.x)*8, off.y+self.hovered_tile_xy.1*8)).show();
                self.gba_obj_arrows[3].set_position(Vector2D::new(off.x+self.hovered_tile_xy.0*8, off.y+selected_tower.bounds.position.y*8 - 8)).show();
                self.gba_obj_arrows[2].set_position(Vector2D::new(off.x+self.hovered_tile_xy.0*8, off.y+(selected_tower.bounds.position.y+selected_tower.bounds.size.y)*8)).show();

            }


            gba_oam.commit();
        }
    }

    pub fn update(&mut self, pf: &PlayingField, gba_vram : &mut VRamManager, gba_oam : &OamManaged) {


        self.gba_obj_hover_anim_counter+=1;
        if self.gba_obj_hover_anim_counter>=10 {
            self.gba_obj_hover_anim_counter=0;
            self.gba_obj_hover_anim_frame = (self.gba_obj_hover_anim_frame+1)%2;
            self.gba_objs_update_req=true;
        }

        self.update_gba_bgs(pf, gba_vram);
        self.update_gba_objs(pf, gba_oam);


        /*push_camera_state();
        set_camera(&self.cam2d);

        let ox: f32 = self.game_res.x * 0.5 - (pf.width as f32) * 0.5 * TOWER_TILE_SIZE;
        let oy: f32 = self.game_res.y * 0.5 - (pf.height as f32) * 0.5 * TOWER_TILE_SIZE;

        // tile bgs
        let color_bg = Color::new(1.0, 1.0, 1.0, 0.5);
        for x in 0..pf.width {
            for y in 0..pf.height {

                if let Some(m) = &pf.mask {
                    if !m.mask[lin2d!(x,y,pf.width)] {
                        continue;
                    }
                }

                let tx: f32 = ox + (x as f32) * TOWER_TILE_SIZE;
                let ty: f32 = oy + (y as f32) * TOWER_TILE_SIZE;

                draw_texture_ex(
                    &assets.texs.towers,
                    tx,
                    ty,
                    color_bg,
                    DrawTextureParams {
                        source: Some(Rect::new(53.0, 1.0, TOWER_TILE_SIZE, TOWER_TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }
        }

        // towers
        let mut text_color = Color::from_rgba(255, 255, 255, 200);
        for x in 0..pf.width {
            for y in 0..pf.height {
                let lin_idx = lin2d!(x, y, pf.width);

                let tx: f32 = ox + (x as f32) * TOWER_TILE_SIZE;
                let mut ty: f32 = oy + (y as f32) * TOWER_TILE_SIZE;

                let tower_index: i32 = pf.field[lin_idx];
                if tower_index != NO_TOWER {
                    let tower = &pf.towers[tower_index as usize];

                    let is_tower_origin: bool = x == tower.x && y == tower.y;

                    if is_tower_origin {
                        //offset y if tower is of certain height
                        ty -= min(20, tower.flattened_height-1) as f32 * 0.5;
                    } else {
                        //offset y if tile are animated after flattening
                        let tas = &self.tile_anim_state[lin_idx];
                        let fay = if tas.timer_flatten >= 0.0 && tas.timer_flatten <= 0.2 {
                            f32::sin(tas.timer_flatten / 0.2 * PI) * -TOWER_TILE_SIZE * 0.75
                        } else {
                            0.0
                        };
                        ty += fay;
                    }

                    let mut tower_color = self.tower_colors.0[tower_index as usize % self.tower_colors.0.len()];
                    if self.blend_to_final_tower_color.enabled {
                        let k = self.blend_to_final_tower_color.blend*pf.area as f32 >= lin_idx as f32;
                        if let Some(m) = &pf.mask {
                            let final_color = m.final_colors[lin_idx];
                            /*tower_color.r = tower_color.r*(1.0-self.blend_to_final_tower_color.blend) + final_color.r*self.blend_to_final_tower_color.blend;
                            tower_color.g = tower_color.g*(1.0-self.blend_to_final_tower_color.blend) + final_color.g*self.blend_to_final_tower_color.blend;
                            tower_color.b = tower_color.b*(1.0-self.blend_to_final_tower_color.blend) + final_color.b*self.blend_to_final_tower_color.blend;*/
                            if k {
                                tower_color = final_color;
                            }
                        }

                        //text_color.a = 200.0/255.0 * (1.0-self.blend_to_final_tower_color.blend);
                        text_color.a = if k {
                            0.0
                        } else {
                            200.0 / 255.0
                        };
                    }

                    // towers underneath
                    if is_tower_origin {
                        let underneath_height = min(20, tower.flattened_height-1) / 2;
                        
                        draw_texture_ex(
                            &assets.texs.towers,
                            tx,
                            ty+TOWER_TILE_SIZE-2.0,
                            tower_color,
                            DrawTextureParams {
                                source: Some(Rect::new(27.0,  1.0 + TOWER_TILE_SIZE - underneath_height as f32 - 2.0,
                                     TOWER_TILE_SIZE, underneath_height as f32 + 2.0)),
                                ..Default::default()
                            },
                        );
                    }

                    // tower
                    draw_texture_ex(
                        &assets.texs.towers,
                        tx,
                        ty,
                        tower_color,
                        DrawTextureParams {
                            source: Some(Rect::new(1.0, 1.0, TOWER_TILE_SIZE, TOWER_TILE_SIZE)),
                            ..Default::default()
                        },
                    );

                    // number / height on tower
                    if is_tower_origin {
                        let mut fh: i32 = tower.flattened_height;
                        if fh >= 100 {
                            let n: i32 = fh / 100;
                            fh -= n * 100;
                        }
                        let nox: f32 =
                            TOWER_TILE_SIZE * 0.5 - (fh.to_string().len() as f32 * 8.0 * 0.5);
                        let noy: f32 = 5.0;
                        let mut nx: f32 = tx + nox;
                        let ny = ty + noy;
                        //digit 2
                        if fh >= 10 {
                            let n: i32 = fh / 10;
                            fh -= n * 10;

                            draw_texture_ex(
                                &assets.texs.numbers,
                                nx,
                                ny,
                                text_color,
                                DrawTextureParams {
                                    source: Some(Rect::new(
                                        1.0 + (n as f32) * 10.0,
                                        1.0,
                                        8.0,
                                        12.0,
                                    )),
                                    ..Default::default()
                                },
                            );

                            nx += 7.0;
                        }
                        //digit 1
                        {
                            draw_texture_ex(
                                &assets.texs.numbers,
                                nx,
                                ny,
                                text_color,
                                DrawTextureParams {
                                    source: Some(Rect::new(
                                        1.0 + (fh as f32) * 10.0,
                                        1.0,
                                        8.0,
                                        12.0,
                                    )),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }

        // possibly added tiles
        let old_flatten_preview_count = self.flatten_preview_count;
        self.flatten_preview_count = 0;
        if let Some((x, y)) = self.selected_tower_xy {
            if let Some((hx, hy)) = self.hovered_tile_xy {
                if let Some(fxys) = pf.calc_flatten_xys((x, y), (hx, hy)) {
                    self.flatten_preview_count = fxys.len();

                    let selected_tower_index: i32 =
                        pf.field[lin2d!(x as usize, y as usize, pf.width)];

                    let mut tower_color = self.tower_colors.0[selected_tower_index as usize % self.tower_colors.0.len()];
                    tower_color.a=0.5;

                    for fxy in fxys.iter() {
                        let ptx: f32 = ox + (fxy.0 as f32) * TOWER_TILE_SIZE;
                        let pty: f32 = oy + (fxy.1 as f32) * TOWER_TILE_SIZE;

                        draw_texture_ex(
                            &assets.texs.towers,
                            ptx,
                            pty,
                            tower_color,
                            DrawTextureParams {
                                source: Some(Rect::new(1.0, 1.0, TOWER_TILE_SIZE, TOWER_TILE_SIZE)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        if self.flatten_preview_count != old_flatten_preview_count {
            play_sound_once(&assets.snds.flatten_preview);
        }

        // selection
        if let Some((x, y)) = self.selected_tower_xy {
            let selected_tower_index: i32 = pf.field[lin2d!(x as usize, y as usize, pf.width)];

            let tx: f32 = ox + (x as f32) * TOWER_TILE_SIZE;
            let ty: f32 = oy + (y as f32) * TOWER_TILE_SIZE;

            // non origin tower tiles
            let mut first = true;
            let mut last;
            for x2 in 0..pf.width as i32 {
                let tower_index: i32 = pf.field[lin2d!(x2 as usize, y as usize, pf.width)];
                let next_tower_index: i32 = if x2 + 1 < (pf.width as i32) {
                    pf.field[lin2d!((x2 + 1) as usize, y as usize, pf.width)]
                } else {
                    -1
                };
                if selected_tower_index == tower_index && x2 != x {
                    last = tower_index != next_tower_index;

                    let tx2: f32 = ox + (x2 as f32) * TOWER_TILE_SIZE;

                    let sx: f32 = 96.0
                        + if first && x2 < x {
                            32.0
                        } else {
                            if last && x2 > x {
                                64.0
                            } else {
                                0.0
                            }
                        };
                    let sy: f32 = 96.0;

                    draw_texture_ex(
                        &assets.texs.sel,
                        tx2 - 4.0,
                        ty - 4.0,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(
                                sx,
                                sy,
                                TOWER_TILE_SIZE + 8.0,
                                TOWER_TILE_SIZE + 8.0,
                            )),
                            ..Default::default()
                        },
                    );

                    if first {
                        first = false;
                    }
                }
            }

            first = true;
            for y2 in 0..pf.height as i32 {
                let tower_index: i32 = pf.field[lin2d!(x as usize, y2 as usize, pf.width)];
                let next_tower_index: i32 = if y2 + 1 < (pf.height as i32) {
                    pf.field[lin2d!(x as usize, (y2 + 1) as usize, pf.width)]
                } else {
                    -1
                };
                if selected_tower_index == tower_index && y2 != y {
                    last = tower_index != next_tower_index;

                    let ty2: f32 = oy + (y2 as f32) * TOWER_TILE_SIZE;

                    let sx: f32 = 0.0
                        + if first && y2 < y {
                            32.0
                        } else {
                            if last && y2 > y {
                                64.0
                            } else {
                                0.0
                            }
                        };
                    let sy: f32 = 96.0;

                    draw_texture_ex(
                        &assets.texs.sel,
                        tx - 4.0,
                        ty2 - 4.0,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(
                                sx,
                                sy,
                                TOWER_TILE_SIZE + 8.0,
                                TOWER_TILE_SIZE + 8.0,
                            )),
                            ..Default::default()
                        },
                    );

                    if first {
                        first = false;
                    }
                }
            }

            // origin tower tile
            let af: i32 = max(0, min(12, ((get_time() * 15.0) % 13.0) as i32));
            let sx: f32 = (af % 8) as f32 * 32.0;
            let sy: f32 = 32.0 + (af / 8) as f32 * 32.0;

            draw_texture_ex(
                &assets.texs.sel,
                tx - 4.0,
                ty - 4.0 - min(20, pf.towers[selected_tower_index as usize].flattened_height-1) as f32 * 0.5,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(
                        sx,
                        sy,
                        TOWER_TILE_SIZE + 8.0,
                        TOWER_TILE_SIZE + 8.0,
                    )),
                    ..Default::default()
                },
            );
        }

        // hover
        if let Some((x, y)) = self.hovered_tower_xy {
            if self.selected_tower_xy != self.hovered_tower_xy {
                let tx: f32 = ox + (x as f32) * TOWER_TILE_SIZE;
                let ty: f32 = oy + (y as f32) * TOWER_TILE_SIZE;

                let sx: f32 = if get_time() % 0.4 > 0.2 { 0.0 } else { 32.0 };
                draw_texture_ex(
                    &assets.texs.sel,
                    tx - 4.0,
                    ty - 4.0,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            sx,
                            0.0,
                            TOWER_TILE_SIZE + 8.0,
                            TOWER_TILE_SIZE + 8.0,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    

        pop_camera_state();*/
    }

    pub fn handle_input(&mut self, pf: &mut PlayingField, gba_input : &ButtonControllerAutoRepeat, gba_mixer : &mut Mixer, game_settings : &Settings) {

        if self.input_mode == PlayingFieldViewInputMode::MoveSelect {

            let mut cursor_moved = false;
            if gba_input.is_just_pressed_or_auto_repeated(Button::UP) && self.hovered_tile_xy.1>0 {
                self.hovered_tile_xy.1-=1;
                if(self.hovered_tile_xy.1==pf.height as i32/2){
                    self.gba_bg_menu_update_req=true;
                }
                self.gba_objs_update_req=true;
                cursor_moved = true;
            }
            else if gba_input.is_just_pressed_or_auto_repeated(Button::DOWN) && self.hovered_tile_xy.1<pf.height as i32 - 1 {
                self.hovered_tile_xy.1+=1;
                if(self.hovered_tile_xy.1==pf.height as i32/2+1){
                    self.gba_bg_menu_update_req=true;
                }
                self.gba_objs_update_req=true;
                cursor_moved = true;
            }

            if gba_input.is_just_pressed_or_auto_repeated(Button::LEFT) && self.hovered_tile_xy.0>0 {
                self.hovered_tile_xy.0-=1;
                self.gba_objs_update_req=true;
                cursor_moved = true;
            }
            else if gba_input.is_just_pressed_or_auto_repeated(Button::RIGHT) && self.hovered_tile_xy.0<pf.width as i32 - 1 {
                self.hovered_tile_xy.0+=1;
                self.gba_objs_update_req=true;
                cursor_moved = true;
            }

            if game_settings.sound && cursor_moved {
                let mut sc = SoundChannel::new(CURSOR_MOVE_SOUND);
                sc.stereo();
                gba_mixer.play_sound(sc);
            }


            if gba_input.btn_ctrl.is_just_pressed(Button::A.union(Button::B))  {
                let tower_idx = pf.field[lin2d!(self.hovered_tile_xy.0 as usize,self.hovered_tile_xy.1 as usize,pf.width)];
                if tower_idx!=NO_TOWER {
                    let tower = &pf.towers[tower_idx as usize];
                    self.selected_tower_xy = Some((tower.x as i32, tower.y as i32));
                    self.hovered_tile_xy = (tower.x as i32, tower.y as i32);
                    self.input_mode = if gba_input.btn_ctrl.is_just_pressed(Button::A) {PlayingFieldViewInputMode::Flatten} else {PlayingFieldViewInputMode::Deflatten};

                    
                    if game_settings.sound {
                        let mut sc = SoundChannel::new(SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }
        } else {
            let mut evtl_dir = None;
            if gba_input.is_just_pressed_or_auto_repeated(Button::UP) {
                evtl_dir = Some((0, -1));
            } else if gba_input.is_just_pressed_or_auto_repeated(Button::DOWN) {
                evtl_dir = Some((0, 1));
            } else if gba_input.is_just_pressed_or_auto_repeated(Button::LEFT) {
                evtl_dir = Some((-1, 0));
            } else if gba_input.is_just_pressed_or_auto_repeated(Button::RIGHT) {
                evtl_dir = Some((1, 0));
            }

            if let Some(dir) = evtl_dir {
                if self.input_mode==PlayingFieldViewInputMode::Flatten {
                    if let Some(fxys) = pf.calc_flatten_towards_xys(self.selected_tower_xy.unwrap(), dir, 1) {
                        let solved_before = pf.is_solved();
                        pf.flatten(self.selected_tower_xy.unwrap(), fxys);
                        self.gba_bg_tiles_and_nums_update_req = true;
                        if game_settings.sound {
                            let mut sc = SoundChannel::new(FLATTEN_DEFLATTEN_SOUND);
                            sc.stereo();
                            gba_mixer.play_sound(sc);
                        }
                        if !solved_before && pf.is_solved() {
                            self.gba_bg_menu_update_req=true;
                            
                            if game_settings.sound {
                                let mut sc = SoundChannel::new(SOLVED_SOUND);
                                sc.stereo();
                                gba_mixer.play_sound(sc);
                            }
                        }
                    }
                } else if self.input_mode==PlayingFieldViewInputMode::Deflatten {
                    if let Some(fxys) = pf.calc_deflatten_towards_xys(self.selected_tower_xy.unwrap(), (-dir.0, -dir.1), 1) {
                        let solved_before = pf.is_solved();
                        pf.deflatten(self.selected_tower_xy.unwrap(), fxys);
                        self.gba_bg_tiles_and_nums_update_req = true;
                        if game_settings.sound {
                            let mut sc = SoundChannel::new(FLATTEN_DEFLATTEN_SOUND);
                            sc.stereo();
                            gba_mixer.play_sound(sc);
                        }
                        if solved_before && !pf.is_solved() {
                            self.gba_bg_menu_update_req=true;
                        }
                    }
                }
            }

            if gba_input.btn_ctrl.is_just_pressed(Button::A.union(Button::B)) {
                if (self.input_mode==PlayingFieldViewInputMode::Flatten && gba_input.btn_ctrl.is_just_pressed(Button::A)) ||
                    (self.input_mode==PlayingFieldViewInputMode::Deflatten && gba_input.btn_ctrl.is_just_pressed(Button::B)) {
                    //cancel flatten/deflatten mode
                    self.input_mode = PlayingFieldViewInputMode::MoveSelect;
                    self.selected_tower_xy = None;
                    
                    if game_settings.sound {
                        let mut sc = SoundChannel::new(SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }

                } else {
                    //change flatten/deflatten mode
                    self.input_mode = if self.input_mode==PlayingFieldViewInputMode::Flatten {PlayingFieldViewInputMode::Deflatten} else {PlayingFieldViewInputMode::Flatten};
                    
                    if game_settings.sound {
                        let mut sc = SoundChannel::new(SELECT_SOUND);
                        sc.stereo();
                        gba_mixer.play_sound(sc);
                    }
                }
            }

        }

            
        if gba_input.btn_ctrl.is_just_pressed(Button::START.union(Button::SELECT)) {
            self.exit_mode = if pf.is_solved() {IngameExitMode::Exit_BoardCompleted} else {IngameExitMode::Exit_BoardNotCompleted};
            self.gba_objs_update_req=true;
            self.gba_bg_bg_update_req=true;
            self.gba_bg_tiles_and_nums_update_req=true;
            
            if game_settings.sound {
                let mut sc = SoundChannel::new(SELECT_SOUND);
                sc.stereo();
                gba_mixer.play_sound(sc);
            }
        }
    }


}
