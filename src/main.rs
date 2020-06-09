use std::rc::Rc;
use std::cell::RefCell;

use gtk::{Application, ApplicationWindow, DrawingArea, Builder};
use gdk::EventMask;
use gtk::prelude::*;
use gio::prelude::*;

use pizarra::App;
use pizarra::point::Point;

fn connect_drawing_area(builder: &Builder, controller: Rc<RefCell<App>>) -> DrawingArea {
    let drawing_area: DrawingArea = builder.get_object("drawing-area").expect("No drawing_area");

    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK;

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

    let copy = controller.clone();
    drawing_area.connect_size_allocate(move |dw, allocation| copy.borrow_mut().on_size_allocate(dw, *allocation));

    drawing_area
}

fn init(app: &Application) {
    // Initialize layout from .glade file
    let builder = Builder::new_from_file("res/layout.glade");
    let controller = Rc::new(RefCell::new(App::new(Point::new(1.0, 1.0))));
    let window: ApplicationWindow = builder.get_object("main-window").expect("Couldn't get window");

    window.set_application(Some(app));

    connect_drawing_area(&builder, controller.clone());

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
