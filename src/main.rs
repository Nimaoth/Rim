use sdl2;

fn main() {
    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("Game", 900, 700)
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
        ui.show_demo_window(&mut true);

        // imgui::Window::new(im_str!("Test"))
        //     .size([400, 400], Condition::Always)
        //     .build(imgui::Ui, f)

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
