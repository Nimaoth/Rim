use sdl2;

use std::boxed::Box;
use std::path::*;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::sync::mpsc;

use notify::{Watcher, watcher};

use super::view::{View, FilterMethod};
use super::image::Image;
use super::layout::{Layout, GridLayout};

// use imgui::im_str;

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

    show_titlebars  : bool,

    selected        : usize,
}

impl App {
    pub fn new() -> App {
        let sdl = sdl2::init().unwrap();
        
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Game", 1000, 900)
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
        imgui.style_mut().window_rounding = 0.0;
        imgui.style_mut().window_border_size = 1.0;

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
            
            show_titlebars  : false,

            selected        : 0,
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

        if self.views.len() > 0 {
            self.views[0].selected = true;
        }

        let mut event_pump = self.sdl.event_pump().unwrap();
        'main: loop {
            for event in event_pump.poll_iter() {
                self.imgui_sdl2.handle_event(&mut self.imgui, &event);
                if self.imgui_sdl2.ignore_event(&event) {
                    continue;
                }

                // println!("{:?}", event);

                use sdl2::event::Event;
                match event {
                    // quit
                    Event::Quit { .. } => break 'main,
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                        break 'main;
                    },

                    // toggle titlebar
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::LAlt), repeat : false, .. } => {
                        self.show_titlebars = !self.show_titlebars;
                    },

                    // reload
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::R), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::R), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } |
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::F5), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].reload();
                        }
                    },

                    // filter method
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].set_filter_menthod(FilterMethod::Linear);
                        }
                    },
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::N), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::N), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].set_filter_menthod(FilterMethod::Nearest);
                        }
                    },

                    // switch selection
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::Tab), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].selected = false;
                            self.selected = ((self.selected - 1) % self.views.len() + self.views.len()) % self.views.len();
                            self.views[self.selected].selected = true;
                        }
                    },
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::Tab), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].selected = false;
                            self.selected = (self.selected + 1) % self.views.len();
                            self.views[self.selected].selected = true;
                        }
                    },

                    Event::KeyDown {  scancode: Some(sdl2::keyboard::Scancode::I), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].selected = false;
                            self.selected = self.layout.get_next_index(self.views.len(), self.selected, 0, -1);
                            self.views[self.selected].selected = true;
                        }
                    },
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::K), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected].selected = false;
                            self.selected = self.layout.get_next_index(self.views.len(), self.selected, 0, 1);
                            self.views[self.selected].selected = true;
                        }
                    },
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::J), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected as usize].selected = false;
                            self.selected = self.layout.get_next_index(self.views.len(), self.selected, -1, 0);
                            self.views[self.selected as usize].selected = true;
                        }
                    },
                    Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), .. } => {
                        if self.selected < self.views.len() {
                            self.views[self.selected as usize].selected = false;
                            self.selected = self.layout.get_next_index(self.views.len(), self.selected, 1, 0);
                            self.views[self.selected as usize].selected = true;
                        }
                    },

                    //
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
            // ui.show_demo_window(&mut true);

            let window_size = self.window.size();

            if self.views.len() > 0 {
                self.layout.layout(&mut self.views, window_size.0 as i32, window_size.1 as i32);
            }

            for view in self.views.iter_mut() {
                view.render(&ui, self.show_titlebars);
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
