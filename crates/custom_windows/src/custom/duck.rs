use crate::{Window, SharedState};
use portal2_sdk::Engine;
use egui::{Context, ColorImage, Color32};
use std::time::Instant;

static OBJ_DATA: &str = include_str!("duck.obj");

#[path = "rendering.rs"]
mod rendering;

use crate::custom::duck::rendering::WIDTH as WIDTH;
use crate::custom::duck::rendering::HEIGHT as HEIGHT;
use crate::custom::duck::rendering::Model as Model;
use crate::custom::duck::rendering::Rasterizer as Rasterizer;

#[path = "obj_parser.rs"]
mod obj_parser;

pub struct MyWindow {
    is_open: bool,
    time: Instant,

    x_input: String,
    y_input: String,
    z_input: String,

    xrot_input: String,
    // No Y rotation since that is hardcoded to constantly spin at 2 radians / second
    //yrot_input: String,
    zrot_input: String,

    render: Option<egui::TextureHandle>,
    render_image: Option<ColorImage>,
    model: Option<Model>,
}

impl Default for MyWindow {
    fn default() -> Self {
        Self {
            time: Instant::now(),
            is_open: false,

            x_input: "0.0".to_string(),//String::new(),
            y_input: "-25.0".to_string(),//String::new(),
            z_input: "100.0".to_string(),//String::new(),

            xrot_input: "0.0".to_string(),//String::new(),
            //yrot_input: "0.0".to_string(),//String::new(),
            zrot_input: "-1.57079632679".to_string(),//String::new(),

            render: None,
            render_image: Some(ColorImage::new([WIDTH, HEIGHT], Color32::BLACK)),
            model: None,
        }
    }
}

fn parse_f32_with_fallback(input: &str, fallback: f32) -> f32 {
    input.trim().parse::<f32>().unwrap_or(fallback)
}

impl Window for MyWindow {
    fn name(&self) -> &'static str { "Spinning Duck" }
    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    // Optional: Only draw when overlay is focused
    fn is_should_render(&self, shared_state: &SharedState, _engine: &portal2_sdk::Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, _shared: &mut SharedState, _engine: &Engine) {
        egui::Window::new(self.name())
            .open(&mut self.is_open)
            .resizable(true)
            .show(ctx, |ui| {
                let options = egui::TextureOptions {
                    //magnification: egui::TextureFilter::Nearest,
                    //minification: egui::TextureFilter::Nearest,
                    ..Default::default()
                };

                let texture: &mut egui::TextureHandle = self.render.get_or_insert_with(|| {
                    // Load the texture only once.
                    ui.ctx().load_texture("render", egui::ColorImage::example(), options)
                });

                let model: &mut Model = self.model.get_or_insert_with(|| {
                    // Load the model only once.
                    obj_parser::parse_obj(OBJ_DATA)
                });


                ui.horizontal(|ui| {
                    ui.label("Position xyz:");
                    ui.text_edit_singleline(&mut self.x_input);
                    ui.text_edit_singleline(&mut self.y_input);
                    ui.text_edit_singleline(&mut self.z_input);
                    model.x = parse_f32_with_fallback(&self.x_input, model.x);
                    model.y = parse_f32_with_fallback(&self.y_input, model.y);
                    model.z = parse_f32_with_fallback(&self.z_input, model.z);
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation xz:");
                    ui.text_edit_singleline(&mut self.xrot_input);
                    //ui.text_edit_singleline(&mut self.yrot_input);
                    ui.text_edit_singleline(&mut self.zrot_input);
                    model.xrot = parse_f32_with_fallback(&self.xrot_input, model.xrot);
                    //model.yrot = parse_f32_with_fallback(&self.yrot_input, model.yrot);
                    model.yrot = self.time.elapsed().as_secs_f32() * -2.0;
                    model.zrot = parse_f32_with_fallback(&self.zrot_input, model.zrot);
                });


                let render_image = self.render_image.as_mut().unwrap();
                let mut rasterizer = Rasterizer { image: render_image, depth_buffer: &mut Box::new([0.0; WIDTH * HEIGHT]) };
                for item in rasterizer.depth_buffer.iter_mut() {
                    *item = f32::MAX;
                }
                
                rasterizer.clear();
                model.render(&mut rasterizer);

                texture.set(render_image.clone(), options);
                ui.image(&*texture);
            });
    }
}