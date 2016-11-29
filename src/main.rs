//! A demonstration using glutin to provide events and glium for drawing the Ui.
//!
//! Note that the `glium` crate is re-exported via the `conrod::backend::glium` module.

#[macro_use] extern crate conrod;

#[cfg(not(target_os="android"))]
extern crate find_folder;
#[cfg(target_os="android")]
extern crate android_glue;

extern crate image;
extern crate rusttype;

mod support;

use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

// The initial width and height in "points".
const WIN_W: u32 = support::WIN_W;
const WIN_H: u32 = support::WIN_H;

#[cfg(not(target_os="android"))]
fn load_asset<P: AsRef<std::path::Path>>(path: P) -> std::io::Cursor<Vec<u8>> {
  use std::io::Read;

  let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
  let mut file = std::fs::File::open(&assets.join(path)).unwrap();
  let mut buf = Vec::new();
  file.read_to_end(&mut buf).unwrap();
  std::io::Cursor::new(buf)
}

#[cfg(target_os="android")]
fn load_asset<P: AsRef<std::path::Path>>(path: P) -> std::io::Cursor<Vec<u8>> {
  let filename = path.as_ref().to_str().expect("Cannot convert Path to &str");
  match android_glue::load_asset(filename) {
    Ok(buf) => std::io::Cursor::new(buf),
    Err(_) => panic!("Couldn't load asset '{}'", filename),
  }
}
pub fn main() {

    // Build the window.
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(WIN_W, WIN_H)
        .with_title("Conrod with glium!")
        .with_gl(glium::glutin::GlRequest::Specific(glium::glutin::Api::OpenGlEs, (3, 0)))
        .build_glium()
        .unwrap();

    // A demonstration of some app state that we want to control with the conrod GUI.
    let mut app = support::DemoApp::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();

    // The `widget::Id` of each widget instantiated in `support::gui`.
    let ids = support::Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let font = rusttype::FontCollection::from_bytes(load_asset("fonts/NotoSans/NotoSans-Regular.ttf").into_inner()).into_font().unwrap();
    ui.fonts.insert(font);

    // Load the Rust logo from our assets folder to use as an example image.
    fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
        let rgba_image = image::load(load_asset("images/rust.png"), image::ImageFormat::PNG).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(rgba_image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
        texture
    }

    let image_map = support::image_map(&ids, load_rust_logo(&display));

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    //
    // Internally, the `Renderer` maintains:
    // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
    // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
    // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
    // `conrod::render::Primitive`s.
    // - a `Vec` of commands that describe how to draw the vertices.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // Start the loop:
    //
    // - Render the current state of the `Ui`.
    // - Update the widgets via the `support::gui` fn.
    // - Poll the window for available events.
    // - Repeat.
    'main: loop {

        // Poll for events.
        for event in display.poll_events() {

            // Use the `glutin` backend feature to convert the glutin event to a conrod one.
            let window = display.get_window().unwrap();
            if let Some(event) = conrod::backend::glutin::convert(event.clone(), window) {
                ui.handle_event(event);
            }

            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                glium::glutin::Event::Closed =>
                    break 'main,

                _ => {},
            }
        }

        // We must manually track the window width and height as it is currently not possible to
        // receive `Resize` events from glium on Mac OS any other way.
        //
        // TODO: Once the following PR lands, we should stop tracking size like this and use the
        // `window_resize_callback`. https://github.com/tomaka/winit/pull/88
        if let Some(win_rect) = ui.rect_of(ui.window) {
            let (win_w, win_h) = (win_rect.w() as u32, win_rect.h() as u32);
            let (w, h) = display.get_window().unwrap().get_inner_size_points().unwrap();
            if w != win_w || h != win_h {
                let event = conrod::event::Input::Resize(w, h);
                ui.handle_event(event);
            }
        }

        // If some input event has been received, update the GUI.
        if ui.global_input.events().next().is_some() {
            // Instantiate a GUI demonstrating every widget type provided by conrod.
            let mut ui = ui.set_widgets();
            support::gui(&mut ui, &ids, &mut app);
        }

        // Draw the `Ui`.
        if let Some(primitives) = ui.draw_if_changed() {
            // The issue is somewhere in this block of code

            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(1.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }

        // Avoid hogging the CPU.
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
