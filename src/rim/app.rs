use sdl2;

use std::boxed::Box;
use std::path::*;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::sync::mpsc;

use notify::{Watcher, watcher};

use super::view::{View, FilterMethod};
use super::image::Image;
use super::layout::{Layout, GridLayout, LayoutDirection};
use super::open_file_dialog::OpenFileDialog;
use super::util::*;

pub struct App {
    views           : Vec<View>,
    layout          : Box<dyn Layout>,
    layout_direction: LayoutDirection,
    auto_layout_dir : bool,

    sdl             : sdl2::Sdl,
    _video_subsystem: sdl2::VideoSubsystem,
    window          : sdl2::video::Window,
    _gl_context     : sdl2::video::GLContext,
    imgui           : imgui::Context,
    imgui_sdl2      : imgui_sdl2::ImguiSdl2,
    opengl_renderer : imgui_opengl_renderer::Renderer,

    dir_watcher     : notify::RecommendedWatcher,
    dir_watcher_recv: mpsc::Receiver<notify::DebouncedEvent>,

    show_titlebars  : bool,

    selected        : usize,

    maximized       : bool,
    open_file_dialog: OpenFileDialog,

    error_msg       : Option<String>,
}

impl App {
    pub fn new(floating: bool, width: i32, height: i32) -> App {
        let sdl = sdl2::init().unwrap();
        
        let video_subsystem = sdl.video().unwrap();
        let mut window = video_subsystem.window("Rim", width as u32, height as u32);
        window.opengl();
        
        if !floating {
            window.resizable();
        }
            
        let window = window.build().unwrap();

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
        let watcher = watcher(watch_send, Duration::from_millis(500)).unwrap();

        App {
            views           : Vec::new(),
            layout          : GridLayout::new(),
            layout_direction: LayoutDirection::Vertical,
            auto_layout_dir : true,

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
            maximized       : false,

            open_file_dialog: OpenFileDialog::new(),

            error_msg       : None,
        }
    }

    pub fn open_image(&mut self, path: &Path) -> Result<(), ()> {
        println!("open {:?}", path);
        let path = get_absolute_path(path);

        match self.find_image_by_path(&path) {
            Some(_) => Ok(()),
            None => {
                println!("Opening image {:?}", path);

                match Image::new(&path) {
                    Ok(image) => {
                        let mut view = View::new();
                        view.images.push(image.clone());
                        self.views.push(view);
                        self.dir_watcher.watch(path, notify::RecursiveMode::NonRecursive).unwrap_or(());
                        Ok(())
                    },
                    Err(msg) => {
                        self.error_msg = Some(msg);
                        Err(())
                    },
                }
            },
        }
    }

    fn find_image_by_path(&mut self, path: &Path) -> Option<&Image> {
        for view in self.views.iter() {
            for image in view.images.iter() {
                if image.path == path {
                    return Some(image);
                }
            }
        }

        None
    }

    pub fn run(&mut self) {
        if self.views.len() > 0 {
            self.views[0].selected = true;
        }

        let mut event_pump = self.sdl.event_pump().unwrap();
        'main: loop {
            for event in event_pump.poll_iter() {
                if self.open_file_dialog.is_open() {
                    use sdl2::event::Event;
                    match event {
                        // quit
                        Event::Quit { .. } => break 'main,
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            break 'main;
                        },

                        //
                        _ => {
                            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
                        }
                    }
                } else {
                    use sdl2::event::Event;
                    match event {
                        // reload
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::R), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::R), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::F5), .. } => {
                            if self.selected < self.views.len() {
                                match self.views[self.selected].reload() {
                                    Err(msg) => self.error_msg = Some(msg),
                                    Ok(_) => {},
                                }
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

                        // layout direction 
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::H), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::H), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            self.auto_layout_dir = false;
                            self.layout_direction = LayoutDirection::Horizontal;
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::V), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::V), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            self.auto_layout_dir = false;
                            self.layout_direction = LayoutDirection::Vertical;
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::A), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::A), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            self.auto_layout_dir = true;
                        },
                        
                        // close selected
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::W), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::W), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            if self.selected < self.views.len() && self.views.len() > 1 {
                                self.views.remove(self.selected);
                                if !self.views.is_empty() {
                                    self.selected = self.selected % self.views.len();
                                    self.views[self.selected].selected = true;
                                }
                            }
                        },

                        // open new
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::O), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::O), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            let path = if self.selected < self.views.len() {
                                let sel_path: &Path = &self.views[self.selected].images[0].path;
                                match sel_path.parent() {
                                    Some(parent) => parent.to_str().unwrap().to_owned(),
                                    None => sel_path.to_str().unwrap().to_owned(),
                                }
                            } else {
                                "/home".to_owned()
                            };
                            self.open_file_dialog.open(path);
                        },
                        
                        // move selected
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::I), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::I), keymod: sdl2::keyboard::Mod::RSHIFTMOD, .. } => {
                            if self.selected < self.views.len() {
                                let new_selected = self.layout.get_next_index(self.views.len(), self.selected, 0, -1, self.layout_direction);
                                self.views.swap(self.selected, new_selected);
                                self.selected = new_selected;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::K), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::K), keymod: sdl2::keyboard::Mod::RSHIFTMOD, .. } => {
                            if self.selected < self.views.len() {
                                let new_selected = self.layout.get_next_index(self.views.len(), self.selected, 0, 1, self.layout_direction);
                                self.views.swap(self.selected, new_selected);
                                self.selected = new_selected;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::J), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::J), keymod: sdl2::keyboard::Mod::RSHIFTMOD, .. } => {
                            if self.selected < self.views.len() {
                                let new_selected = self.layout.get_next_index(self.views.len(), self.selected, -1, 0, self.layout_direction);
                                self.views.swap(self.selected, new_selected);
                                self.selected = new_selected;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), keymod: sdl2::keyboard::Mod::RSHIFTMOD, .. } => {
                            if self.selected < self.views.len() {
                                let new_selected = self.layout.get_next_index(self.views.len(), self.selected, 1, 0, self.layout_direction);
                                self.views.swap(self.selected, new_selected);
                                self.selected = new_selected;
                            }
                        },

                        // maximize
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::M), keymod: sdl2::keyboard::Mod::LCTRLMOD, .. } |
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::M), keymod: sdl2::keyboard::Mod::RCTRLMOD, .. } => {
                            if self.maximized {
                                self.window.set_fullscreen(sdl2::video::FullscreenType::Off).unwrap_or(());
                            } else {
                                self.window.set_fullscreen(sdl2::video::FullscreenType::True).unwrap_or(());
                            }
                            self.maximized = self.window.fullscreen_state() == sdl2::video::FullscreenType::True;
                        },

                        // switch selection
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::Tab), keymod: sdl2::keyboard::Mod::LSHIFTMOD, .. } => {
                            if self.selected < self.views.len() {
                                self.views[self.selected].selected = false;
                                self.selected =(self.selected + self.views.len() - 1) % self.views.len();
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
                                self.selected = self.layout.get_next_index(self.views.len(), self.selected, 0, -1, self.layout_direction);
                                self.views[self.selected].selected = true;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::K), .. } => {
                            if self.selected < self.views.len() {
                                self.views[self.selected].selected = false;
                                self.selected = self.layout.get_next_index(self.views.len(), self.selected, 0, 1, self.layout_direction);
                                self.views[self.selected].selected = true;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::J), .. } => {
                            if self.selected < self.views.len() {
                                self.views[self.selected as usize].selected = false;
                                self.selected = self.layout.get_next_index(self.views.len(), self.selected, -1, 0, self.layout_direction);
                                self.views[self.selected as usize].selected = true;
                            }
                        },
                        Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::L), .. } => {
                            if self.selected < self.views.len() {
                                self.views[self.selected as usize].selected = false;
                                self.selected = self.layout.get_next_index(self.views.len(), self.selected, 1, 0, self.layout_direction);
                                self.views[self.selected as usize].selected = true;
                            }
                        },

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

                        //
                        _ => {
                            self.imgui_sdl2.handle_event(&mut self.imgui, &event);
                        }
                    }
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

            // auto layout
            if self.auto_layout_dir {
                // calc average aspect ratio

                let mut aspect = 0.0;
                let mut count = 0.0;
                for view in self.views.iter() {
                    for img in view.images.iter() {
                        aspect += img.width as f32 / img.height as f32;
                        count += 1.0;
                    }
                }

                let avg = aspect / count;

                let window_size = self.window.drawable_size();
                self.layout_direction = if (window_size.0 as f32 / window_size.1 as f32) > avg {
                    LayoutDirection::Horizontal
                } else {
                    LayoutDirection::Vertical
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
                self.layout.layout(&mut self.views, window_size.0 as i32, window_size.1 as i32, self.layout_direction);
            }

            let view_count = self.views.len();
            for view in self.views.iter_mut() {
                let border_color = match (view.selected, view_count) {
                    (true, 1) =>  [0.2, 0.2, 0.2, 1.0],
                    (true, _) =>  [1.0, 1.0, 1.0, 1.0],
                    (false, _) => [0.2, 0.2, 0.2, 1.0],
                };
                let tok = ui.push_style_color(imgui::StyleColor::Border, border_color);
                view.render(&ui, self.show_titlebars, !self.open_file_dialog.is_open());
                tok.pop(&ui);
            }

            let file_to_open = self.open_file_dialog.render(&ui, self.window.drawable_size());

            // error message
            let mut show_err = false;
            match &self.error_msg {
                Some(msg) => {
                    show_err = true;
                    imgui::Window::new(imgui::im_str!("Error"))
                        .focus_on_appearing(true)
                        .focused(true)
                        .bring_to_front_on_focus(true)
                        .opened(&mut show_err)
                        .position([0.0, 0.0], imgui::Condition::Always)
                        .scroll_bar(false)
                        .scrollable(false)
                        .always_auto_resize(true)
                        .collapsible(false)
                        .build(&ui, || {
                            ui.text(msg);
                        });
                },
                None => {},
            }

            if !show_err {
                self.error_msg = None;
            }

            unsafe {
                gl::ClearColor(0.3, 0.3, 0.5, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            
            // render window contents here
            self.imgui_sdl2.prepare_render(&ui, &self.window);
            self.opengl_renderer.render(ui);
            self.window.gl_swap_window();

            match file_to_open {
                Some(file_to_open) => {
                    self.open_image(&file_to_open).unwrap_or(());
                },
                None => {},
            }
        }
    }
}
