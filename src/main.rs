use sdl2;

#[macro_use]
extern crate clap;

use std::rc::Rc;
use imgui::im_str;

// #[derive(Clap)]
// #[clap(version = "1.0", author = "Nimaoth")]
// struct Opts {
//     /// Sets a custom config file. Could have been an Option<T> with no default too
//     #[clap(short, long, default_value = "default.conf")]
//     config: String,
//     /// Some input. Because this isn't an Option<T> it's required to be used
//     input: String,
//     /// A level of verbosity, and can be used multiple times
//     #[clap(short, long, parse(from_occurrences))]
//     verbose: i32,
//     #[clap(subcommand)]
//     subcmd: SubCommand,
// }


macro_rules! GL {
    ($expression:expr) => {
        unsafe {
            use gl::*;
            $expression;
            loop {
                let gl_error = gl::GetError();
                if gl_error == gl::NO_ERROR {
                    break;
                }

                println!("[OpenGL] Error: {}", gl_error);
            }
        }
    };
}
clap::arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Foo {
        Bar,
        Baz,
        Qux
    }
}

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Nimaoth")
        (about: "View images")
        (@arg file: -f +takes_value "Display this file")
        (@arg directory: -d +takes_value "Display files in this folder")
    ).get_matches();

    let mut images = Vec::new();

    if matches.is_present("file") {
        let image_path = matches.value_of("file").unwrap();

        match image::open(image_path) {
            Ok(img) => images.push(img),
            Err(msg) => eprintln!("Failed to load image '{}': {}", image_path, msg)
        }
    }
    if matches.is_present("directory") {
        let dir_path = matches.value_of("directory").unwrap();
        match std::fs::read_dir(dir_path) {
            Ok(dir) => {
                for image_path in dir.into_iter() {
                    match image_path {
                        Ok(path) => {
                            let path = path.path().canonicalize().unwrap();
                            match image::open(path.clone()) {
                                Ok(img) => images.push(img),
                                Err(msg) => eprintln!("Failed to load image '{:?}': {}", path, msg)
                            }
                        },
                        Err(msg) => eprintln!("Error: {}", msg)
                    }
                }
            },
            Err(msg) => eprintln!("Failed to load files in directory '{}': {}", dir_path, msg)
        }
    }

    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("Game", 1280, 720)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().expect("Couldn't create GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
    
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video_subsystem.gl_get_proc_address(s) as _);
    
    let mut tex_ids: Rc<Vec<u32>> = Rc::new(Vec::new());

    for img in images.iter() {
        let (img, width, height, format, data_format, data_type) = match img.as_rgb8() {
            Some(rgb) => (rgb.as_ref(), rgb.width(), rgb.height(), gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            None => continue
        };
        let mut tex_id: u32 = 0;
        GL!(GenTextures(1, &mut tex_id));
        GL!(BindTexture(TEXTURE_2D, tex_id));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32));
        GL!(TexImage2D(TEXTURE_2D, 0, format as i32, width as i32, height as i32, 0, data_format, data_type, img.as_ptr() as *const std::ffi::c_void));
        std::rc::Rc::get_mut(&mut tex_ids).unwrap().push(tex_id);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        let ui = imgui.frame();

        let window_size = window.size();
        let tex_ids = tex_ids.clone();

        imgui::Window::new(im_str!("test"))
            .position([0f32, 0f32], imgui::Condition::Always)
            .size([window_size.0 as f32, window_size.1 as f32], imgui::Condition::Always)
            .resizable(false)
            .title_bar(false)
            .build(&ui, || {
                unsafe {
                    for tex_id in &*tex_ids {
                        imgui::Image::new(std::mem::transmute(*tex_id as usize), [500f32, 500f32])
                            .build(&ui);
                    }
                }
                if ui.small_button(im_str!("Press me")) {
                    println!("button pressed")
                }
            });
        

        // ui.show_demo_window(&mut true);

        // ui.text("hi");

        unsafe {
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // render window contents here
        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);
        window.gl_swap_window();
    }
}
