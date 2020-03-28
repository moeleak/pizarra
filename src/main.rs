use opengl_graphics::{ GlGraphics, OpenGL };
use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
#[cfg(feature = "include_sdl2")]
use sdl2_window::Sdl2Window as AppWindow;
#[cfg(feature = "include_glfw")]
use glfw_window::GlfwWindow as AppWindow;
#[cfg(feature = "include_glutin")]
use glutin_window::GlutinWindow as AppWindow;
use std::fs::File;
use std::io::Write;
use chrono::Local;

use pizarra::{App, ShapeType};
use pizarra::color::Color;

fn main() -> std::io::Result<()> {
    let opengl = OpenGL::V3_2;
    let initial_dimentions = [800.0, 400.0];

    // piston stuff
    let mut window: AppWindow = WindowSettings::new("Pizarra", initial_dimentions)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut events = Events::new(EventSettings::new().lazy(true));

    let gl = GlGraphics::new(opengl);
    let mut app = App::new(gl, initial_dimentions);
    let mut ctrl_on = false;
    let mut alt_on = false;

    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            app.render(&args);
        }

        match event.press_args() {
            Some(Button::Mouse(MouseButton::Left)) => app.start_drawing(),
            Some(Button::Mouse(MouseButton::Middle)) => app.start_moving(),
            Some(Button::Keyboard(Key::LCtrl)) | Some(Button::Keyboard(Key::RCtrl)) => { ctrl_on = true; },
            Some(Button::Keyboard(Key::LAlt)) | Some(Button::Keyboard(Key::RAlt)) => { alt_on = true; },
            Some(Button::Keyboard(Key::Z)) => if ctrl_on {
                app.undo();
            },
            Some(Button::Keyboard(Key::R)) => {
                if ctrl_on {
                    app.set_tool(ShapeType::Rectangle);
                }
                if alt_on {
                    app.set_color(Color::red());
                }
            },
            Some(Button::Keyboard(Key::B)) => if alt_on {
                app.set_color(Color::blue());
            },
            Some(Button::Keyboard(Key::G)) => if alt_on {
                app.set_color(Color::green());
            },
            Some(Button::Keyboard(Key::Y)) => if alt_on {
                app.set_color(Color::yellow());
            },
            Some(Button::Keyboard(Key::O)) => if alt_on {
                app.set_color(Color::orange());
            },
            Some(Button::Keyboard(Key::W)) => if alt_on {
                app.set_color(Color::white());
            },
            Some(Button::Keyboard(Key::L)) => if ctrl_on {
                app.set_tool(ShapeType::Line);
            },
            Some(Button::Keyboard(Key::C)) => if ctrl_on {
                app.set_tool(ShapeType::Circle);
            },
            Some(Button::Keyboard(Key::NumPadPlus)) => app.zoom_in(),
            Some(Button::Keyboard(Key::NumPadMinus)) => app.zoom_out(),
            Some(Button::Keyboard(Key::D0)) => app.go_home(),
            _ => {},
        }

        match event.release_args() {
            Some(Button::Mouse(MouseButton::Left)) => app.finish_drawing(),
            Some(Button::Mouse(MouseButton::Middle)) => app.end_moving(),
            Some(Button::Keyboard(Key::LCtrl)) | Some(Button::Keyboard(Key::RCtrl)) => {
                ctrl_on = false;
            },
            Some(Button::Keyboard(Key::LAlt)) | Some(Button::Keyboard(Key::RAlt)) => {
                alt_on = false;
            },
            _ => {},
        }

        event.mouse_cursor(|coords| {
            app.handle_cursor(coords);
        });
        event.mouse_relative(|coords| {
            app.handle_cursor_relative(coords);
        });
        event.mouse_scroll(|[dx, dy]| {
            app.delta_offset([dx, -dy]);
        });
        event.resize(|resize_args| {
            app.resize(resize_args.window_size);
        });
    }

    // save the result file
    let filename = Local::now().format("pizarra_%Y-%m-%dT%H-%M-%S.svg").to_string();
    let mut file = File::create(filename)?;

    file.write_all(&app.serialize().into_bytes())?;

    Ok(())
}
