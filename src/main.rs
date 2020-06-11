use std::rc::Rc;
use std::cell::RefCell;

use gtk::{
    Application, ApplicationWindow, DrawingArea, Builder, ColorButton,
    Button,
};
use gdk::{EventMask, EventType};
use gtk::prelude::*;
use gio::prelude::*;

use pizarra::{App, app::ShouldRedraw, color::Color};
use pizarra::point::Point;

fn init(app: &Application) {
    // Initialize layout from .glade file
    let layout = include_str!("../res/layout.glade");
    let builder = Builder::new_from_string(layout);
    let controller = Rc::new(RefCell::new(App::new(Point::new(1.0, 1.0))));
    let window: ApplicationWindow = builder.get_object("main-window").expect("Couldn't get window");

    window.set_application(Some(app));

    let drawing_area: DrawingArea = builder.get_object("drawing-area").expect("No drawing_area");

    // Drawing area
    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK;

    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    let copy = controller.clone();
    drawing_area.connect_draw(move |dw, ctx| copy.borrow_mut().render(dw, ctx));

    let copy = controller.clone();
    drawing_area.connect_button_press_event(move |_dw, event| {
        if let EventType::ButtonPress = event.get_event_type() {
            match event.get_button() {
                1 => copy.borrow_mut().start_drawing(),
                2 => copy.borrow_mut().start_moving(),
                _ => {},
            }
        }

        Inhibit(false)
    });

    let copy = controller.clone();
    drawing_area.connect_button_release_event(move |_dw, event| {
        if let EventType::ButtonRelease = event.get_event_type() {
            match event.get_button() {
                1 => copy.borrow_mut().end_drawing(),
                2 => copy.borrow_mut().end_moving(),
                _ => {},
            }
        }

        Inhibit(false)
    });

    let copy = controller.clone();
    drawing_area.connect_motion_notify_event(move |dw, event| {
        let (x, y) = event.get_position();

        if let ShouldRedraw::Yes = copy.borrow_mut().handle_cursor(Point::new(x, y)) {
            dw.queue_draw();
        }

        Inhibit(false)
    });

    let copy = controller.clone();
    drawing_area.connect_size_allocate(move |_dw, allocation| {
        copy.borrow_mut().resize(Point::new(allocation.width as f64, allocation.height as f64))
    });

    let boxed_dw = Rc::new(RefCell::new(drawing_area));

    // Color chooser
    let color_chooser: ColorButton = builder.get_object("color-chooser").expect("No color chooser");

    let copy = controller.clone();
    let dwb = boxed_dw.clone();
    color_chooser.connect_color_set(move |chooser| {
        copy.borrow_mut().set_color(Color::from(chooser.get_rgba()));
        dwb.borrow().queue_draw();
    });

    // Zoom buttons
    let zoom_in_btn: Button = builder.get_object("zoom-in-btn").expect("No zoom in btn");
    let copy = controller.clone();
    let dwb = boxed_dw.clone();
    zoom_in_btn.connect_clicked(move |_btn| {
        copy.borrow_mut().zoom_in();
        dwb.borrow().queue_draw();
    });

    let zoom_out_btn: Button = builder.get_object("zoom-out-btn").expect("No zoom out btn");
    let copy = controller.clone();
    let dwb = boxed_dw.clone();
    zoom_out_btn.connect_clicked(move |_btn| {
        copy.borrow_mut().zoom_out();
        dwb.borrow().queue_draw();
    });

    let zoom_home_btn: Button = builder.get_object("zoom-home-btn").expect("No zoom home btn");
    let copy = controller.clone();
    let dwb = boxed_dw.clone();
    zoom_home_btn.connect_clicked(move |_btn| {
        copy.borrow_mut().go_home();
        dwb.borrow().queue_draw();
    });

    // Show
    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("tk.categulario.pizarra"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| init(app));

    application.run(&[]);
}
