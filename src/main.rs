use sdl2;

#[macro_use]
extern crate clap;

use imgui::im_str;
use std::boxed::Box;
use std::path::*;
use std::rc::Rc;

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

trait Layout {
    fn layout(&self, views: &mut [View], width: i32, height: i32);
}

struct GridLayout {}

impl GridLayout {
    fn new() -> Box<dyn Layout> {
        Box::new(GridLayout {})
    }
}

impl Layout for GridLayout {
    fn layout(&self, views: &mut [View], width: i32, height: i32) {
        
    }
}

struct View {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    images: Vec<Rc<Image>>,
}

impl View {
    fn new() -> View {
        View {
            x       : 0,
            y       : 0,
            width   : 400,
            height  : 400,
            images  : Vec::new()
        }
    }

    fn render(&self, ui: &imgui::Ui) {
        let title: String = self.images[0].path.to_str().unwrap().to_owned();
        let title = im_str!("{}", title);
        imgui::Window::new(&title)
            // .position([self.x as f32, self.y as f32], imgui::Condition::Always)
            // .size(
            //     [self.width as f32, self.height as f32],
            //     imgui::Condition::Always,
            // )
            // .resizable(false)
            // .title_bar(false)
            .build(&ui, || {
                if ui.small_button(im_str!("Reload from disk")) {
                    for img in self.images.iter() {
                        img.reload_from_disk();
                    }
                }
                unsafe {
                    for img in self.images.iter() {
                        imgui::Image::new(std::mem::transmute(img.renderer_id), [self.width as f32, self.height as f32])
                            .build(&ui);
                    }
                }
            });
    }
}

struct Image {
    path: std::path::PathBuf,
    renderer_id: usize,
}

impl Image {
    fn new(path: &Path) -> Result<Image, ()> {
        let image = match image::open(path) {
            Ok(img) => img,
            Err(_) => return Err(()),
        };

        let (img_data, width, height, format, data_format, data_type) = match image.as_rgb8() {
            Some(rgb) => (
                rgb.as_ref(),
                rgb.width(),
                rgb.height(),
                gl::RGB8,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            None => return Err(()),
        };

        let mut tex_id: u32 = 0;
        GL!(GenTextures(1, &mut tex_id));
        GL!(BindTexture(TEXTURE_2D, tex_id));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32));
        GL!(TexParameteri(
            TEXTURE_2D,
            TEXTURE_MAG_FILTER,
            NEAREST as i32
        ));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32));
        GL!(TexImage2D(
            TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            data_format,
            data_type,
            img_data.as_ptr() as *const std::ffi::c_void
        ));
        Ok(Image {
            path: path.to_owned(),
            renderer_id: tex_id as usize,
        })
    }

    fn reload_from_disk(&self) -> Result<(), ()> {
        let image = match image::open(&self.path) {
            Ok(img) => img,
            Err(_) => return Err(()),
        };

        let (img_data, width, height, format, data_format, data_type) = match image.as_rgb8() {
            Some(rgb) => (
                rgb.as_ref(),
                rgb.width(),
                rgb.height(),
                gl::RGB8,
                gl::RGB,
                gl::UNSIGNED_BYTE,
            ),
            None => return Err(()),
        };

        GL!(BindTexture(TEXTURE_2D, self.renderer_id as u32));
        GL!(TexImage2D(
            TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            data_format,
            data_type,
            img_data.as_ptr() as *const std::ffi::c_void
        ));

        Ok(())
    }
}

struct App {
    views: Vec<View>,
    layout: Box<dyn Layout>,
    images: Rc<Vec<Rc<Image>>>,

    sdl: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    imgui: imgui::Context,
    imgui_sdl2: imgui_sdl2::ImguiSdl2,
    opengl_renderer: imgui_opengl_renderer::Renderer,
}

impl App {
    fn new() -> App {
        let sdl = sdl2::init().unwrap();

        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Game", 1280, 720)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });

        App {
            views: Vec::new(),
            layout: GridLayout::new(),
            images: Rc::new(Vec::new()),

            sdl: sdl,
            video_subsystem: video_subsystem,
            window: window,
            gl_context: gl_context,
            imgui: imgui,
            imgui_sdl2: imgui_sdl2,
            opengl_renderer: renderer,
        }
    }

    fn open_image(&mut self, path: &Path) -> Result<(), ()> {
        match Image::new(path) {
            Ok(image) => Ok(Rc::get_mut(&mut self.images).unwrap().push(Rc::new(image))),
            Err(_) => Err(()),
        }
    }

    fn run(&mut self) {
        if self.views.len() == 0 {
            for image in self.images.iter() {
                let mut view = View::new();
                view.images.push(image.clone());
                self.views.push(view);
            }
        }

        let mut event_pump = self.sdl.event_pump().unwrap();
        'main: loop {
            for event in event_pump.poll_iter() {
                self.imgui_sdl2.handle_event(&mut self.imgui, &event);
                if self.imgui_sdl2.ignore_event(&event) {
                    continue;
                }
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    _ => {}
                }
            }

            self.imgui_sdl2.prepare_frame(
                self.imgui.io_mut(),
                &self.window,
                &event_pump.mouse_state(),
            );

            let ui = self.imgui.frame();

            let window_size = self.window.size();

            self.layout.layout(&mut self.views, window_size.0 as i32, window_size.1 as i32);

            for view in self.views.iter() {
                view.render(&ui);
            }

            // ui.show_demo_window(&mut true);
            unsafe {
                gl::ClearColor(0.3, 0.3, 0.5, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            // render window contents here
            self.imgui_sdl2.prepare_render(&ui, &self.window);
            self.opengl_renderer.render(ui);
            self.window.gl_swap_window();
        }
    }
}

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Nimaoth")
        (about: "View images")
        (@arg file: -f +takes_value "Display this file")
        (@arg directory: -d +takes_value "Display files in this folder")
    )
    .get_matches();

    let mut app = App::new();

    if matches.is_present("file") {
        let image_path = matches.value_of("file").unwrap();
        match app.open_image(Path::new(image_path)) {
            Ok(img) => {}
            Err(_) => eprintln!("Failed to load image '{}'", image_path),
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
                            match app.open_image(&path) {
                                Ok(img) => {}
                                Err(_) => eprintln!("Failed to load image '{:?}'", &path),
                            }
                        }
                        Err(msg) => eprintln!("Error: {}", msg),
                    }
                }
            }
            Err(msg) => eprintln!("Failed to load files in directory '{}': {}", dir_path, msg),
        }
    }

    app.run();
}
