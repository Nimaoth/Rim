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
    pub x           : i32,
    pub y           : i32,
    pub width       : i32,
    pub height      : i32,
    pub images      : Vec<Rc<Image>>,

    filter_method   : FilterMethod,

    rect_pos        : Vec2,
    zoom            : f32,

    pan_speed       : f32,
    zoom_speed      : f32,

    pub selected    : bool,
}

impl View {
    pub fn new() -> View {
        View {
            x               : 0,
            y               : 0,
            width           : 400,
            height          : 400,
            images          : Vec::new(),

            filter_method   : FilterMethod::Nearest,

            rect_pos        : Vec2::zero(),
            zoom            : 1.0,

            pan_speed       : 7.5,
            zoom_speed      : 3.0,

            selected        : false,
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui, title_bar: bool, focus: bool) {
        let title: String = self.images[0].path.to_str().unwrap().to_owned();
        let title = im_str!("{}", title);

        let tok = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
        defer!(tok.pop(ui));


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
                ui.menu_bar(|| {
                    imgui::MenuItem::new(im_str!("File"))
                        .build(ui);
                });
                
                let context_menu_name = im_str!("View Context Menu {:?}", self.images[0].path);
                let tok = ui.push_style_var(imgui::StyleVar::WindowPadding([7.0, 7.0]));
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
                            self.set_filter_menthod(self.filter_method);
                        }
                    }

                });

                if ui.is_mouse_hovering_rect(
                    ui.window_pos(), 
                    (Vec2::from(ui.window_pos()) + Vec2::from(ui.window_size())).into()) && ui.is_mouse_down(imgui::MouseButton::Right) {
                    ui.open_popup(&context_menu_name);
                }
                tok.pop(ui);

                
                let [content_region_width, content_region_height] = ui.content_region_avail();
                let content_region_as = content_region_width / content_region_height;

                if ui.is_window_focused() && self.selected && !ui.is_key_down(sdl2::keyboard::Scancode::Application as u32) {
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
                }

                for img in self.images.iter() {
                    let image_as = img.width as f32 / img.height as f32;
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
                        continue;
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
                    if rect_max.x >= content_region_width - border {
                        uv1.x = 1.0 - (rect_max.x - content_region_width + border) / rect_size.x;
                        rect_max.x = content_region_width - border - 1.0;
                    }
                    if rect_max.y >= content_region_height - border {
                        uv1.y = 1.0 - (rect_max.y - content_region_height + border) / rect_size.y;
                        rect_max.y = content_region_height - border - 1.0;
                    }

                    let pos = rect_min.into();
                    let size = rect_max - rect_min;
                    ui.set_cursor_pos(pos);
                    unsafe {
                        imgui::Image::new(std::mem::transmute(img.renderer_id), [size.x, size.y])
                            .uv0(uv0.into())
                            .uv1(uv1.into())
                            .build(&ui);
                    }
                }
            });
    }

    pub fn set_filter_menthod(&mut self, filter_method: FilterMethod) {
        self.filter_method = filter_method;
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

    pub fn reload(&self) {
        for img in self.images.iter() {
            img.reload_from_disk().unwrap_or(());
        }
    }
}
