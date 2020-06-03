use std::rc::Rc;
use imgui::im_str;

use super::image::Image;
use super::vec::Vec2;

fn clamp(f: f32, min: f32, max: f32) -> f32 {
    if f < min {
        return min;
    } else if f > max {
        return max;
    } else {
        return f;
    }
}

struct Defer<F>
    where F : FnOnce() {
    func: Option<F>
}

impl<F> Drop for Defer<F> where F : FnOnce() {
    fn drop(&mut self) {
        match self.func.take() {
            Some(func) => (func)(),
            None => {},
        }
    }
}

macro_rules! defer {
    ($expression:expr) => {
        let _d = Defer{func: Some(|| { $expression })};
    };
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum FilterMethod {
    Nearest,
    Linear
}

pub struct View {
    id              : u32,
    pub x           : i32,
    pub y           : i32,
    pub width       : i32,
    pub height      : i32,
    pub image       : Rc<Image>,

    pub filter_method : FilterMethod,

    rect_pos        : Vec2,
    zoom            : f32,

    pan_speed       : f32,
    zoom_speed      : f32,

    pub selected    : bool,

    frozen              : bool,
    pub history_enabled : bool,
}

impl View {
    pub fn new(id: u32, image: Rc<Image>, enable_history: bool) -> View {
        View {
            id              : id,
            x               : 0,
            y               : 0,
            width           : 400,
            height          : 400,
            image           : image,

            filter_method   : FilterMethod::Nearest,

            rect_pos        : Vec2::zero(),
            zoom            : 1.0,

            pan_speed       : 7.5,
            zoom_speed      : 3.0,

            selected        : false,

            frozen          : false,
            history_enabled : enable_history,
        }
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn render(&mut self, ui: &imgui::Ui, title_bar: bool, focus: bool) -> bool {
        let title: String = self.image.path.to_str().unwrap().to_owned();
        let title = if self.frozen {
            im_str!("{} - past##{}", title, self.id)
        } else {
            im_str!("{}##{}", title, self.id)
        };

        let tok = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
        defer!(tok.pop(ui));

        let mut was_selected = false;

        imgui::Window::new(&title)
            .focus_on_appearing(false)
            .focused(self.selected && focus)
            .position([self.x as f32, self.y as f32], imgui::Condition::Always)
            .size(
                [self.width as f32, self.height as f32],
                imgui::Condition::Always,
            )
            .resizable(false)
            .collapsible(false)
            .title_bar(title_bar)
            .always_use_window_padding(true)
            .scroll_bar(false)
            .scrollable(false)
            .build(&ui, || {
                if ui.is_mouse_hovering_rect(
                    ui.window_pos(), 
                    (Vec2::from(ui.window_pos()) + Vec2::from(ui.window_size())).into())
                    && (ui.is_mouse_clicked(imgui::MouseButton::Right) || ui.is_mouse_clicked(imgui::MouseButton::Left)) {
                    was_selected = true;
                }

                let content_region_max = ui.content_region_max();
                let [content_region_width, content_region_height] = ui.content_region_avail();
                let content_region_as = content_region_width / content_region_height;
                let win = ui.is_key_down(sdl2::keyboard::Scancode::Application as u32);
                let ctrl = ui.is_key_down(sdl2::keyboard::Scancode::LCtrl as u32) || ui.is_key_down(sdl2::keyboard::Scancode::RCtrl as u32);
                let shift = ui.is_key_down(sdl2::keyboard::Scancode::LShift as u32) || ui.is_key_down(sdl2::keyboard::Scancode::RShift as u32);
                
                if ui.is_window_focused() && self.selected && !win {
                    if !ctrl {
                        if ui.is_key_pressed(sdl2::keyboard::Scancode::Space as u32) {
                            self.zoom = 1.0;
                            self.rect_pos = Vec2::zero();
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::W as u32) {
                            self.rect_pos = self.rect_pos + Vec2::new(0.0, -self.pan_speed / self.zoom);
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::S as u32) {
                            self.rect_pos = self.rect_pos + Vec2::new(0.0, self.pan_speed / self.zoom);
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::A as u32) {
                            self.rect_pos = self.rect_pos + Vec2::new(-self.pan_speed / self.zoom, 0.0);
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::D as u32) {
                            self.rect_pos = self.rect_pos + Vec2::new(self.pan_speed / self.zoom, 0.0);
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::Period as u32) {
                            self.zoom *= 1.0 + self.zoom_speed * 0.01;
                        }
                        if ui.is_key_down(sdl2::keyboard::Scancode::Comma as u32) {
                            self.zoom /= 1.0 + self.zoom_speed * 0.01;
                        }
                        if shift {
                            if ui.is_key_down(sdl2::keyboard::Scancode::Up as u32) {
                                self.zoom *= 1.0 + self.zoom_speed * 0.01;
                            }
                            if ui.is_key_down(sdl2::keyboard::Scancode::Down as u32) {
                                self.zoom /= 1.0 + self.zoom_speed * 0.01;
                            }
                        } else {
                            if ui.is_key_down(sdl2::keyboard::Scancode::Up as u32) {
                                self.rect_pos = self.rect_pos + Vec2::new(0.0, -self.pan_speed / self.zoom);
                            }
                            if ui.is_key_down(sdl2::keyboard::Scancode::Down as u32) {
                                self.rect_pos = self.rect_pos + Vec2::new(0.0, self.pan_speed / self.zoom);
                            }
                            if ui.is_key_down(sdl2::keyboard::Scancode::Left as u32) {
                                self.rect_pos = self.rect_pos + Vec2::new(-self.pan_speed / self.zoom, 0.0);
                            }
                            if ui.is_key_down(sdl2::keyboard::Scancode::Right as u32) {
                                self.rect_pos = self.rect_pos + Vec2::new(self.pan_speed / self.zoom, 0.0);
                            }
                        }
                    }
                }

                let image_as = self.image.width as f32 / self.image.height as f32;
                let (width, height) = if image_as > content_region_as {
                    (content_region_width, content_region_width / image_as)
                } else {
                    (content_region_height * image_as, content_region_height)
                };


                self.rect_pos.x = clamp(self.rect_pos.x, -width * 0.5, width * 0.5);
                self.rect_pos.y = clamp(self.rect_pos.y, -height * 0.5, height * 0.5);

                // center
                let center_pos = Vec2::new(0.0, 0.0);
                let camspace = center_pos - self.rect_pos * self.zoom;
                let viewspace = camspace + Vec2::new(content_region_width, content_region_height) * 0.5;
                let mut rect_min = viewspace - Vec2::new(width, height) * 0.5 * self.zoom;
                let mut rect_max = viewspace + Vec2::new(width, height) * 0.5 * self.zoom;
                let rect_size = rect_max - rect_min;

                let mut uv0 = Vec2::zero();
                let mut uv1 = Vec2::new(1.0, 1.0);
                
                if rect_max.x <= 0.0 || rect_max.y <= 0.0 || rect_min.x >= content_region_width || rect_min.y >= content_region_height {
                    return;
                }

                let border = 0.0;

                if rect_min.x < border {
                    uv0.x = -(rect_min.x - border) / rect_size.x;
                    rect_min.x = border;
                }
                if rect_min.y < border {
                    uv0.y = -(rect_min.y - border) / rect_size.y;
                    rect_min.y = border;
                }
                if rect_max.x >= content_region_max[0] - border {
                    uv1.x = 1.0 - (rect_max.x - content_region_max[0] + border) / rect_size.x;
                    rect_max.x = content_region_max[0] - border - 1.0;
                }
                if rect_max.y >= content_region_max[1] - border {
                    uv1.y = 1.0 - (rect_max.y - content_region_max[1] + border) / rect_size.y;
                    rect_max.y = content_region_max[1] - border - 1.0;
                }

                let pos = (rect_min + ui.window_content_region_min().into()).into();
                let size = rect_max - rect_min;
                ui.set_cursor_pos(pos);
                unsafe {
                    imgui::Image::new(std::mem::transmute(self.image.renderer_id), [size.x, size.y])
                        .uv0(uv0.into())
                        .uv1(uv1.into())
                        .build(&ui);
                }
            });

        return was_selected;
    }

    pub fn set_filter_menthod(&mut self, filter_method: FilterMethod) {
        self.filter_method = filter_method;
        GL!(BindTexture(TEXTURE_2D, self.image.renderer_id as u32));
        
        let filter_method = match self.filter_method {
            FilterMethod::Linear => gl::LINEAR,
            FilterMethod::Nearest => gl::NEAREST,
        } as i32;
        
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, filter_method));
        GL!(TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, filter_method));
        GL!(BindTexture(TEXTURE_2D, 0));
    }

    pub fn reload(&self) -> Result<(), String> {
        if !self.frozen {
            self.image.reload_from_disk()?;
        }

        Ok(())
    }
}
