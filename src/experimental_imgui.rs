pub struct ImGuiContext {
    platform: crate::experimental_imgui_sdl2_support::SdlPlatform,
    imgui: imgui::Context,
    imgui_renderer: Option<imgui_glow_renderer::Renderer>,
    imgui_renderer_texture_map: imgui_glow_renderer::SimpleTextureMap,
    first_frame: bool,
}

impl ImGuiContext {
    pub fn new() -> Self {
        let mut imgui = imgui::Context::create();
        // #todo(pku-nekomaru) expose imgui font api to public
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        // #todo(pku-nekomaru) expose these two switches to public
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        Self {
            platform: crate::experimental_imgui_sdl2_support::SdlPlatform::init(&mut imgui),
            imgui: imgui,
            imgui_renderer: Option::None,
            imgui_renderer_texture_map: imgui_glow_renderer::SimpleTextureMap::default(),
            first_frame: true,
        }
    }

    pub fn handle_event(&mut self, event: & sdl2::event::Event) -> bool {
        self.platform.handle_event(&mut self.imgui, &event);
        match event {
            sdl2::event::Event::KeyDown {..}
            | sdl2::event::Event::KeyUp {..}
            | sdl2::event::Event::TextEditing {..}
            | sdl2::event::Event::TextInput {..}
            => self.imgui.io().want_capture_keyboard,
            sdl2::event::Event::MouseMotion {..}
            | sdl2::event::Event::MouseButtonDown {..}
            | sdl2::event::Event::MouseButtonUp {..}
            | sdl2::event::Event::MouseWheel {..}
            | sdl2::event::Event::FingerDown {..}
            | sdl2::event::Event::FingerUp {..}
            | sdl2::event::Event::FingerMotion {..}
            | sdl2::event::Event::DollarGesture {..}
            | sdl2::event::Event::DollarRecord {..}
            | sdl2::event::Event::MultiGesture {..}
            => self.imgui.io().want_capture_mouse,
            _ => false,
        }
    }

    pub fn frame_begin(&mut self, window: &crate::platform::Window, gl: &glow::Context) -> &mut imgui::Ui {
        self.platform.prepare_frame(
            & mut self.imgui,
            & window.sdl_window,
            & window.event_pump);
        
        if self.first_frame {
            self.first_frame = false;
            let mut texture_map = imgui_glow_renderer::SimpleTextureMap::default();
            self.imgui_renderer = Option::from(
                imgui_glow_renderer::Renderer::initialize(
                    gl,
                    & mut self.imgui,
                    & mut texture_map, true).unwrap());
        }
        self.imgui.frame()
    }

    pub fn frame_end(&mut self, gl: &glow::Context) -> Result<(), imgui_glow_renderer::RenderError> {
        let draw_data = self.imgui.render();
        self.imgui_renderer.as_mut().unwrap().render(
            gl,
            & self.imgui_renderer_texture_map,
            draw_data
        ).unwrap();
        Ok(())
    }
}
