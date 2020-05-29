use std::rc::Rc;
use imgui::im_str;
use super::image::Image;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum FilterMethod {
    Nearest,
    Linear
}

pub struct View {
    pub x           : i32,
    pub y           : i32,
    pub width       : i32,
    pub height      : i32,
    pub images      : Vec<Rc<Image>>,

    filter_method   : FilterMethod
}

impl View {
    pub fn new() -> View {
        View {
            x       : 0,
            y       : 0,
            width   : 400,
            height  : 400,
            images  : Vec::new(),

            filter_method : FilterMethod::Nearest
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        let title: String = self.images[0].path.to_str().unwrap().to_owned();
        let title = im_str!("{}", title);
        imgui::Window::new(&title)
            .position([self.x as f32, self.y as f32], imgui::Condition::Always)
            .size(
                [self.width as f32, self.height as f32],
                imgui::Condition::Always,
            )
            .resizable(false)
            .collapsible(false)
            .title_bar(true)
            .build(&ui, || {
                ui.menu_bar(|| {
                    imgui::MenuItem::new(im_str!("File"))
                        .build(ui);
                });
                
                let context_menu_name = im_str!("View Context Menu {:?}", self.images[0].path);
                ui.popup(&context_menu_name, || {
                    ui.text(self.images[0].path.to_str().unwrap_or(""));
                    ui.separator();
                    if ui.small_button(im_str!("Reload from disk")) {
                        for img in self.images.iter() {
                            img.reload_from_disk().unwrap_or(());
                        }
                    }

                    
                    if let Some(tok) = ui.begin_menu(im_str!("Sampling Method"), true) {
                        let mut changed = false;
                        changed |= ui.radio_button(im_str!("Nearest"), &mut self.filter_method, FilterMethod::Nearest);
                        changed |= ui.radio_button(im_str!("Linear"), &mut self.filter_method, FilterMethod::Linear);
                        tok.end(ui);

                        if changed {
                            for img in self.images.iter() {
                                GL!(BindTexture(TEXTURE_2D, img.renderer_id as u32));
                                
                                let filter_method = match self.filter_method {
                                    FilterMethod::Linear => gl::LINEAR,
                                    FilterMethod::Nearest => gl::NEAREST,
                                } as i32;
                                
                                GL!(TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, filter_method));
                                GL!(TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, filter_method));
                            }
                            GL!(BindTexture(TEXTURE_2D, 0));
                        }
                    }

                });

                if ui.is_mouse_down(imgui::MouseButton::Right) {
                    ui.open_popup(&context_menu_name);
                }

                
                let [w, h] = ui.content_region_avail();

                unsafe {
                    for img in self.images.iter() {
                        imgui::Image::new(std::mem::transmute(img.renderer_id), [w, h])
                            .build(&ui);
                    }
                }
            });
    }
}

