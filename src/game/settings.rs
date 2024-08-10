extern crate alloc;

use agb::{input::Button, save::SaveData};
use alloc::vec::Vec;

use crate::util::gbaex::ButtonControllerAutoRepeat;

const SETTINGS_HEADER : [u8; 8] = [0xe7, 0x2a, 0xf5, 0x0c, 0x1d, 0x1b, 0x09, 0x18 ];

pub struct Settings {
    pub sound : bool,

    pub playing_field_width: usize,
    pub playing_field_height: usize,

    pub playing_field_seed : [u32; 4],

    pub playing_field_data : Option<Vec<u8>>,

}

impl Settings {
    pub fn new() -> Settings {
        Settings {

            sound : true,

            playing_field_width: 10,
            playing_field_height: 10,
            playing_field_seed: [1014776995, 476057059, 3301633994, 706340607],

            playing_field_data : None,
        }
    }

    pub fn save(&self, save_access : &mut SaveData) -> bool{
        let pfd_len = if let Some(ref pfd) = self.playing_field_data {
            pfd.len()
        } else {
            0
        };
        let data_len = SETTINGS_HEADER.len() + 21;
        let mut data = Vec::<u8>::with_capacity(data_len);
        //write header
        for hb in SETTINGS_HEADER {
            data.push(hb);
        }

        data.push(if self.sound {1} else {0});

        data.push(self.playing_field_width as u8);
        data.push(self.playing_field_height as u8);
        for seed_part in self.playing_field_seed {
            data.push(((seed_part&0xff000000)>>24) as u8);
            data.push(((seed_part&0x00ff0000)>>16) as u8);
            data.push(((seed_part&0x0000ff00)>>8) as u8);
            data.push((seed_part&0x000000ff) as u8);
        }

        let pfd_len_high = ((pfd_len&0xff00)>>8) as u8;
        let pfd_len_low = (pfd_len&0xff) as u8;
        data.push(pfd_len_high);
        data.push(pfd_len_low);

        assert!(data.len()==data_len);

        let pb = save_access.prepare_write(0..(data_len+pfd_len));
        if pb.is_ok() {
            let mut pb = pb.unwrap();
            if pb.write(0, &data).is_err() {
                return false;
            };
            if let Some(ref pfd) = self.playing_field_data {
                if pb.write(data.len(), pfd).is_err() {
                    return false;
                }
            }
        } else {
            return false;
        }

        true

    }

    pub fn load(&mut self, save_access : &mut SaveData) -> bool{
        let data_len = SETTINGS_HEADER.len() + 21;
        let mut data = Vec::<u8>::new();
        data.resize(data_len, 0);
        if save_access.read(0, &mut data).is_ok() {

            let mut di = 0;

            //check for correct header (if not correct, save data is corrupt or no save data where ever saved)
            for (i,hb) in SETTINGS_HEADER.iter().enumerate() {
                if data[i]!=*hb {
                    return false;
                }
            }
            di+=SETTINGS_HEADER.len();

            self.sound = data[di]!=0; di+=1;

            self.playing_field_width = data[di] as usize; di+=1;
            self.playing_field_height = data[di] as usize; di+=1;

            for seed_part in self.playing_field_seed.iter_mut() {
                *seed_part = (data[di+0] as u32)<<24 | (data[di+1] as u32)<<16 | (data[di] as u32)<<8 | (data[di] as u32);
                di+=4;
            }

            self.playing_field_data = None;
            let pfd_len = (data[di+0] as usize)<<8 | (data[di+1] as usize);
            if pfd_len>0 {
                let mut pfd = Vec::<u8>::new();
                pfd.resize(pfd_len, 0);
                if save_access.read(data_len, &mut pfd).is_ok() {
                    self.playing_field_data = Some(pfd);
                }
            }

        }
        true
    }

    pub fn alter_seed_with_input(&mut self, gba_input : &ButtonControllerAutoRepeat) {

        let mut loops = 0;

        if gba_input.is_just_pressed_or_auto_repeated(Button::A) {
            loops+=1;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::B) {
            loops+=2;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::LEFT) {
            loops+=3;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::RIGHT) {
            loops+=4;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::UP) {
            loops+=5;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::DOWN) {
            loops+=6;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::SELECT) {
            loops+=7;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::START) {
            loops+=8;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::R) {
            loops+=9;
        }
        if gba_input.is_just_pressed_or_auto_repeated(Button::L) {
            loops+=10;
        }

        if loops<=0 {
            return;
        }


        //from RandomNumberGenerator::gen()
        for _ in 0..loops {
            let t = self.playing_field_seed[1].wrapping_shr(9);

            self.playing_field_seed[2] ^= self.playing_field_seed[0];
            self.playing_field_seed[3] ^= self.playing_field_seed[1];
            self.playing_field_seed[1] ^= self.playing_field_seed[2];
            self.playing_field_seed[0] ^= self.playing_field_seed[3];

            self.playing_field_seed[2] ^= t;
            self.playing_field_seed[3] = self.playing_field_seed[3].rotate_left(11);
        }
    
            
        //#[allow(arithmetic_overflow)]

    }

    
}