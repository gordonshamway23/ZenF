#![no_std]
#![no_main]

#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod util;
mod assets;
mod game;
mod menu;

//use agb::mgba::{DebugLevel, Mgba};
use agb::sound::mixer::Frequency;


use game::{logic::PlayingField, view::IngameExitMode};
use game::view::PlayingFieldView;
use menu::{MenuExitMode, MenuView};
use util::gbaex::ButtonControllerAutoRepeat;



#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    use agb::save::SaveData;



    let gba_vblank = agb::interrupt::VBlank::get();
    let mut gba_input = ButtonControllerAutoRepeat::new();
    let mut gba_mixer = gba.mixer.mixer(Frequency::Hz32768);
    gba_mixer.enable();

    gba.save.init_sram();
    let mut save_access : Option<SaveData> =  gba.save.access().ok();


    let mut game_settings = game::Settings::new();
    if let Some(ref mut sd) = save_access {
        game_settings.load(sd);
    }

    let mut menu_fmode = MenuExitMode::DoNotExit;
    let mut ingame_fmode = IngameExitMode::DoNotExit;

    loop {

        //menu
        {   
            let (gba_tiled0, mut gba_vram) =  gba.display.video.tiled0();
            let gba_oam = gba.display.object.get_managed();

            let mut mv = MenuView::new(&gba_tiled0, &mut gba_vram, &gba_oam, ingame_fmode == IngameExitMode::DoNotExit);

            loop {
                gba_mixer.frame();

                gba_input.update();

                game_settings.alter_seed_with_input(&gba_input);

                mv.handle_input(&gba_input, &mut gba_mixer, &mut game_settings);
                mv.update(&mut gba_vram, &gba_oam, &mut game_settings);

                gba_vblank.wait_for_vblank();

                if mv.get_exit_mode()!=MenuExitMode::DoNotExit {
                    menu_fmode = mv.get_exit_mode();
                    break;
                }
            }
        }

        //in game
        {
            let (gba_tiled0, mut gba_vram) =  gba.display.video.tiled0();
            let gba_oam = gba.display.object.get_managed();

            let mut pf = PlayingField::new(game_settings.playing_field_width, game_settings.playing_field_height, None);
            let mut pfv = PlayingFieldView::new(&gba_tiled0, &mut gba_vram, &gba_oam);
            if menu_fmode==MenuExitMode::Exit_ContinueGame && game_settings.playing_field_data.is_some() {
                pfv.load_from_u8_vec(&mut pf, game_settings.playing_field_data.as_ref().unwrap());
            } else if menu_fmode==MenuExitMode::Exit_StartNewGame {
                pfv.init_with_random_towers(&mut pf, Some(game_settings.playing_field_seed));
                game_settings.playing_field_data = None;
                if let Some(ref mut sd) = save_access {
                    game_settings.save(sd);
                }
            } else {
                unreachable!();
            }

            //if let Some(mut logger) = Mgba::new() {
            //    let _= logger.print(format_args!("{}",pf.save_as_u8_vec().len()), DebugLevel::Warning); //=> ~ 3KiB for a 30x20 level
            //}

            loop {
                gba_mixer.frame();

                gba_input.update();

                game_settings.alter_seed_with_input(&gba_input);

                pfv.handle_input(&mut pf, &gba_input, &mut gba_mixer, &game_settings);
                pfv.update(&pf, &mut gba_vram, &gba_oam);

                gba_vblank.wait_for_vblank();

                if pfv.get_exit_mode() != IngameExitMode::DoNotExit {
                    ingame_fmode = pfv.get_exit_mode();
                    if ingame_fmode==IngameExitMode::Exit_BoardNotCompleted {
                        game_settings.playing_field_data = Some(pfv.save_as_u8_vec(&pf));
                    } else {
                        game_settings.playing_field_data = None;
                    }
                    if let Some(ref mut sd) = save_access {
                        game_settings.save(sd);
                    }
                    break;
                }
            }  
        }
    }
}