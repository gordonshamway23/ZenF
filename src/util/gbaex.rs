
use agb::input::{ButtonController, Button};

const AUTO_REPEAT_INITIAL_FRAMES : u8 = 20; //frames which must be elapsed before auto repeating starts
const AUTO_REPEAT_INTERVAL_FRAMES : u8 = 4; //after initial phase the frames which must elapse between each repeat

pub struct ButtonControllerAutoRepeat {
    elapsed_frames_while_pressed_per_key : [u8; 10], 
    initial : u32,
    just_auto_repeated : u32,
    pub btn_ctrl : ButtonController,
}

impl ButtonControllerAutoRepeat {
    pub fn new() -> Self {
        ButtonControllerAutoRepeat {
            elapsed_frames_while_pressed_per_key : [0; 10],
            initial : 0,
            just_auto_repeated : 0,
            btn_ctrl : ButtonController::new(),
        }
    }

    pub fn update(&mut self) {
        self.btn_ctrl.update();

        for i in 0..10u32 {
            if self.btn_ctrl.is_pressed(Button::from_bits_retain(1<<i)) {
                self.elapsed_frames_while_pressed_per_key[i as usize] += 1;
                let initial = self.initial&(1<<i)==0;
                let frames = if initial {AUTO_REPEAT_INITIAL_FRAMES} else {AUTO_REPEAT_INTERVAL_FRAMES};
                if self.elapsed_frames_while_pressed_per_key[i as usize] > frames {
                    self.just_auto_repeated |= 1<<i;
                    self.elapsed_frames_while_pressed_per_key[i as usize] = 0;
                    self.initial |= 1<<i;
                } else {
                    self.just_auto_repeated &= !(1<<i);
                }
            } else {
                self.elapsed_frames_while_pressed_per_key[i as usize] = 0;
                self.just_auto_repeated &= !(1<<i);
                self.initial &= !(1<<i);
            }
        }
    }

    pub fn is_just_auto_repeated(&self, keys: Button) -> bool {
        self.just_auto_repeated & keys.bits() == keys.bits()
    }

    pub fn is_just_pressed_or_auto_repeated(&self, keys : Button) -> bool {
        self.btn_ctrl.is_just_pressed(keys) || self.is_just_auto_repeated(keys)
    }
}
