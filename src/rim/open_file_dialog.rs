use imgui::*;
use std::path::*;
use std::fs;

pub struct OpenFileDialog {
    current_dir : String,
    item_list   : Vec<String>,
    selected    : usize,
    open        : bool,
}

impl OpenFileDialog {
    pub fn new() -> OpenFileDialog {
        OpenFileDialog {
            current_dir : String::new(),
            item_list   : Vec::new(),
            selected    : 0,
            open        : false,
        }
    }

    pub fn render(&mut self, ui: &Ui, max_size: (u32, u32)) -> Option<PathBuf> {
        let mut result = None;
        let mut open = self.open;
        use imgui::sys::ImVec2;
        unsafe {
            imgui::sys::igSetNextWindowSize(
                ImVec2::new(std::cmp::min(600, max_size.0) as f32, std::cmp::min(400, max_size.1) as f32),
                imgui::Condition::Always as imgui::sys::ImGuiCond);
        }

        let id = im_str!("{}###OpenFile", &self.current_dir);

        ui.popup_modal(&id)
            .opened(&mut open)
            .movable(true)
            .resizable(true)
            .save_settings(true)
            .always_auto_resize(true)
            .build(|| {
                let mut selected = self.selected;
                if ui.is_window_focused() && !ui.is_key_down(sdl2::keyboard::Scancode::Application as u32) {
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::I as u32) {
                        if !self.item_list.is_empty() {
                            selected = (selected + self.item_list.len() - 1) % self.item_list.len();
                        } else {
                            selected = 0;
                        }
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::K as u32) {
                        if !self.item_list.is_empty() {
                            selected = (selected + 1) % self.item_list.len();
                        } else {
                            selected = 0;
                        }
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::J as u32) {
                        self.move_dir_up();
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::L as u32) {
                        match self.get_selected() {
                            Some(dir) => self.move_dir_down(&dir),
                            None => {}
                        }
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::Return as u32) {
                        result = match self.get_selected() {
                            Some(item) => {
                                let mut path = PathBuf::from(&self.current_dir);
                                path.push(item);
                                Some(path)
                            },
                            None => None,
                        };
                        self.close();
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::Space as u32) {
                        result = match self.get_selected() {
                            Some(item) => {
                                let mut path = PathBuf::from(&self.current_dir);
                                path.push(item);
                                Some(path)
                            },
                            None => None,
                        };
                    }
                    if ui.is_key_pressed(sdl2::keyboard::Scancode::Escape as u32) {
                        self.close();
                    }

                }

                for (i, path) in self.item_list.iter().enumerate() {
                    let p = im_str!("{}", path);
                    if Selectable::new(&p)
                        .selected(i == self.selected)
                        .build(ui) {
                    }
                    if i == selected {
                        ui.set_item_default_focus();
                        unsafe {
                            imgui::sys::igSetScrollHereY(0.5);
                        }
                    }
                }

                self.selected = selected;
            });

        self.open &= open;
        
        if self.open {
            ui.open_popup(&id);
        }

        return result;
    }

    fn get_selected(&self) -> Option<String> {
        if self.selected < self.item_list.len() {
            Some(self.item_list[self.selected].clone())
        } else {
            None
        }
    }

    fn update_list(&mut self){ 
        self.item_list.clear();
        for item in fs::read_dir(PathBuf::from(&self.current_dir).canonicalize().unwrap()).unwrap() {
            match item {
                Ok(item) => {
                    let mut name = item.file_name().to_str().unwrap().to_owned();
                    if item.path().is_dir() {
                        name += "/";
                    }
                    self.item_list.push(name);

                }

                Err(_) => {}
            }
        }
        self.item_list.sort();
        if !self.item_list.is_empty() {
            self.selected %= self.item_list.len();
        } else {
            self.selected = 0;
        }
    }

    fn move_dir_up(&mut self) {
        match PathBuf::from(&self.current_dir).parent() {
            Some(parent) => {
                self.current_dir = parent.to_str().unwrap().to_owned();
                self.update_list();
            }

            None => {}
        }
    }

    fn move_dir_down(&mut self, dir: &str) {
        let mut path = PathBuf::from(&self.current_dir);
        path.push(dir);
        if path.is_dir() {
            self.current_dir = path.to_str().unwrap().to_owned();
            self.update_list();
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self, path: String) {
        self.open = true;
        self.current_dir = path;
        self.update_list();
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}