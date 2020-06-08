use std::rc::Rc;
use std::cell::RefCell;

use gtk::{Application, ApplicationWindow, DrawingArea};
use gdk::EventMask;
use gtk::prelude::*;
use gio::prelude::*;

use pizarra::App;
use pizarra::point::Point;

fn build_drawing_area() -> DrawingArea {
    let drawing_area = DrawingArea::new();
    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK;

    let controller = Rc::new(RefCell::new(App::new(Point::new(800.0, 600.0))));

    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    let copy = controller.clone();
    drawing_area.connect_draw(move |dw, ctx| copy.borrow_mut().render(dw, ctx));

    let copy = controller.clone();
    drawing_area.connect_button_press_event(move |dw, event| copy.borrow_mut().on_button_press(dw, event));

    let copy = controller.clone();
    drawing_area.connect_button_release_event(move |dw, event| copy.borrow_mut().on_button_release(dw, event));

    let copy = controller.clone();
    drawing_area.connect_motion_notify_event(move |dw, event| copy.borrow_mut().on_pointer_move(dw, event));

    drawing_area
}

fn init(app: &Application) {
    let window = ApplicationWindow::new(app);

    window.set_title("Pizarra");
    window.set_default_size(800, 600);

    let drawing_area = build_drawing_area();

    window.add(&drawing_area);

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
