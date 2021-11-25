use std::env;

use crate::input::{handle_key_down, handle_key_up};
use clap::clap_app;
use gameboy::GameBoy;
use gl::types::GLuint;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use sdl2::sys::SDL_GetTicks;

mod input;
mod render;

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

struct Args {
    rom_path: String,
}

fn init_logger() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let enable_log_file: bool = match env::var("RUST_LOG") {
        Ok(val) => val.to_lowercase().contains("debug"),
        Err(_) => false,
    };

    if enable_log_file {
        const LOG_PATTERN: &str = "{m}\n";
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
            .build("log/output.log")
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(LevelFilter::Debug),
            )
            .unwrap();

        log4rs::init_config(config).unwrap();
    } else {
        env_logger::builder().format_timestamp(None).init();
    }

    log_panics::init();
}

fn parse_args() -> Args {
    let matches = clap_app!(partyboy =>
        (version: "1.0")
        (about: "A Gameboy (color?) emulator")
        (@arg rom_path: -r --rom +takes_value +required "The path to the rom to load")
    )
    .get_matches();

    let rom_path = matches.value_of("ROM").unwrap().to_owned();
    Args { rom_path }
}

fn main() {
    #[cfg(debug_assertions)]
    init_logger();

    let args = parse_args();

    let mut gb = GameBoy::new(&args.rom_path);
    log::info!("Initialized gameboy.");

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
    }

    let mut window = video
        .window("Partyboy", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl.event_pump().unwrap();

    let mut fb_id: GLuint = 0;
    let mut tex_id: GLuint = 0;
    render::init_gl_state(&mut tex_id, &mut fb_id);

    unsafe {
        gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    let start_time = unsafe { SDL_GetTicks() } as f32;
    let mut frames: f32 = 0f32;
    let mut elapsed: f32;

    let mut time_since_last_window_update: f32 = 0f32;

    'running: loop {
        use sdl2::event::Event;

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        if let Some(keycode) = keycode {
                            handle_key_down(&mut gb, keycode);
                        }
                    }
                }

                Event::KeyUp {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        if let Some(keycode) = keycode {
                            handle_key_up(&mut gb, keycode);
                        }
                    }
                }

                Event::Quit { .. } => break 'running,

                _ => {}
            }
        }

        for _ in 0..(70_224) {
            gb.tick();
        }

        if gb.consume_draw_flag() {
            render::render_gb(&gb, fb_id, tex_id);

            frames += 1f32;
            elapsed = unsafe { SDL_GetTicks() as f32 - start_time };
            let elapsed_secs = elapsed / 1000.0f32;
            let fps = frames / elapsed_secs;

            if elapsed - time_since_last_window_update > 1000f32 {
                let _ = window.set_title(format!("{:.2}", fps).as_str());
                time_since_last_window_update = elapsed;
            }
        }

        window.gl_swap_window();
    }
}
