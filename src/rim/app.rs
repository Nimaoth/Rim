use sdl2;

use std::boxed::Box;
use std::path::*;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::sync::mpsc;

use notify::{Watcher, watcher};

use super::view::View;
use super::image::Image;
use super::layout::{Layout, GridLayout};

pub struct App {
    views           : Vec<View>,
    layout          : Box<dyn Layout>,
    images          : Rc<Vec<Rc<Image>>>,

    sdl             : sdl2::Sdl,
    _video_subsystem: sdl2::VideoSubsystem,
    window          : sdl2::video::Window,
    _gl_context     : sdl2::video::GLContext,
    imgui           : imgui::Context,
    imgui_sdl2      : imgui_sdl2::ImguiSdl2,
    opengl_renderer : imgui_opengl_renderer::Renderer,

    dir_watcher     : notify::INotifyWatcher,
    dir_watcher_recv: mpsc::Receiver<notify::DebouncedEvent>,
}

impl App {
    pub fn new() -> App {
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
        imgui.style_mut().window_rounding = 0f32;

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });

        let (watch_send, watch_recv) = channel();
        let watcher = watcher(watch_send, Duration::from_secs(1)).unwrap();

        App {
            views           : Vec::new(),
            layout          : GridLayout::new(),
            images          : Rc::new(Vec::new()),

            sdl             : sdl,
            _video_subsystem: video_subsystem,
            window          : window,
            _gl_context     : gl_context,
            imgui           : imgui,
            imgui_sdl2      : imgui_sdl2,
            opengl_renderer : renderer,

            dir_watcher     : watcher,
            dir_watcher_recv: watch_recv,
        }
    }

    pub fn open_image(&mut self, path: &Path) -> Result<(), ()> {
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => return Err(()),
        };

        println!("Opening image {:?}", path);

        match Image::new(&path) {
            Ok(image) => {
                self.dir_watcher.watch(path, notify::RecursiveMode::NonRecursive).unwrap_or(());
                Ok(Rc::get_mut(&mut self.images).unwrap().push(image))
            },
            Err(_) => Err(()),
        }
    }

    fn find_image_by_path(&mut self, path: &Path) -> Option<&Image> {
        for image in self.images.iter() {
            if image.path == path {
                return Some(image);
            }
        }

        None
    }

    pub fn run(&mut self) {
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

            while let Ok(event) = self.dir_watcher_recv.try_recv() {
                println!("{:?}", event);
                match event {
                    notify::DebouncedEvent::NoticeWrite(_) => {},
                    notify::DebouncedEvent::NoticeRemove(_) => {},
                    notify::DebouncedEvent::Create(_) => {},
                    notify::DebouncedEvent::Write(path) => {
                        match self.find_image_by_path(&path) {
                            Some(image) => {
                                image.reload_from_disk().unwrap_or(());
                            }
                            None => {}
                        };
                    },
                    notify::DebouncedEvent::Chmod(_) => {},
                    notify::DebouncedEvent::Remove(_) => {},
                    notify::DebouncedEvent::Rename(_, _) => {},
                    notify::DebouncedEvent::Rescan => {},
                    notify::DebouncedEvent::Error(_, _) => {},
                }
            }

            self.imgui_sdl2.prepare_frame(
                self.imgui.io_mut(),
                &self.window,
                &event_pump.mouse_state(),
            );

            let ui = self.imgui.frame();
            ui.show_demo_window(&mut true);

            let window_size = self.window.size();

            if self.views.len() > 0 {
                self.layout.layout(&mut self.views, window_size.0 as i32, window_size.1 as i32);
            }

            for view in self.views.iter_mut() {
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
