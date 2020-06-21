use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;

use gtk::{
    Application, ApplicationWindow, DrawingArea, Builder, ColorButton,
    Button, MenuItem, FileChooserNative, FileChooserAction, ResponseType,
    HeaderBar, MessageDialog, DialogFlags, MessageType, ButtonsType,
};
use gdk::{EventMask, EventType};
use gtk::prelude::*;
use gio::prelude::*;
use glib::clone;

use pizarra::{
    App, app::{ShouldRedraw, SaveStatus}, color::Color, graphics::Transform,
};
use pizarra::point::Point;

fn set_subtitle(header_bar: &HeaderBar, save_status: &SaveStatus) {
    match save_status {
        SaveStatus::NewAndEmpty => header_bar.set_subtitle(Some("Dibujo vacío")),
        SaveStatus::NewAndChanged => header_bar.set_subtitle(Some("*Dibujo sin guardar")),
        SaveStatus::Unsaved(path) => header_bar.set_subtitle(Some(&format!("*{:?}", path))),
        SaveStatus::Saved(path) => header_bar.set_subtitle(Some(&(format!("{:?}", path)))),
    }
}

fn save_drawing(controller: &App, filename: PathBuf) {
    if let Some([topleft, bottomright]) = controller.get_bounds() {
        let width = (bottomright.x - topleft.x).abs() + 20.0;
        let height = (bottomright.y - topleft.y).abs() + 20.0;
        let bgcolor = Color::black();
        let surface = cairo::SvgSurface::for_stream(width, height, File::create(filename).unwrap()).unwrap();
        let ctx = cairo::Context::new(&surface);
        let t = Transform::new(topleft - Point::new(10.0, 10.0), 1.0);

        ctx.set_source_rgb(bgcolor.r, bgcolor.g, bgcolor.b);
        ctx.paint();

        for cmd in controller.draw_commands([topleft, bottomright]) {
            cmd.draw(&ctx, t);
        }

        surface.finish_output_stream().unwrap();
    }
}

fn init(app: &Application) {
    // Initialize layout from .glade file
    let layout = include_str!("../res/layout.glade");
    let builder = Builder::new_from_string(layout);
    let controller = Rc::new(RefCell::new(App::new(Point::new(1.0, 1.0))));
    let window: ApplicationWindow = builder.get_object("main-window").expect("Couldn't get window");
    let header_bar: HeaderBar = builder.get_object("header-bar").expect("no header bar");

    window.set_application(Some(app));

    // save on exit
    window.connect_delete_event(clone!(@strong controller, @strong window => move |_window, _event| {
        match controller.borrow().get_save_status() {
            SaveStatus::NewAndChanged => {
                let message_dialog = MessageDialog::new(
                    Some(&window),
                    DialogFlags::DESTROY_WITH_PARENT,
                    MessageType::Question,
                    ButtonsType::None,
                    "Ya hiciste algunos dibujos\n\n¿Deseas guardar?",
                );
                message_dialog.add_button("Sí", ResponseType::Yes);
                message_dialog.add_button("No", ResponseType::No);
                message_dialog.add_button("Cancelar", ResponseType::Cancel);

                let response = message_dialog.run();

                message_dialog.hide();

                match response {
                    ResponseType::Yes => {
                        let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(&window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
                        let res = save_file_chooser.run();

                        match res {
                            ResponseType::Accept => {
                                if let Some(filename) = save_file_chooser.get_filename() {
                                    save_drawing(&controller.borrow(), filename);
                                }
                            },
                            x => { println!("{:?}", x); },
                        }

                        Inhibit(false)
                    },
                    ResponseType::Cancel | ResponseType::DeleteEvent => Inhibit(true),
                    _response => Inhibit(false), // "No" and everything else
                }
            },
            _ => Inhibit(false), // in this cases no further saving is required
        }
    }));

    // Drawing area
    let drawing_area: DrawingArea = builder.get_object("drawing-area").expect("No drawing_area");

    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK;

    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    drawing_area.connect_draw(clone!(@strong controller => move |_dw, ctx| {
        let controller = controller.borrow();
        let t = controller.get_transform();
        let bgcolor = Color::black();

        // the bbox of the visible things
        let (corner_1, corner_2) = controller.visible_extent();

        let commands = controller.draw_commands([corner_1, corner_2]);

        ctx.set_source_rgb(bgcolor.r, bgcolor.g, bgcolor.b);
        ctx.paint();

        // content
        for cmd in commands {
            cmd.draw(ctx, t);
        }

        Inhibit(false)
    }));

    drawing_area.connect_button_press_event(clone!(@strong controller => move |_dw, event| {
        if let EventType::ButtonPress = event.get_event_type() {
            match event.get_button() {
                1 => controller.borrow_mut().start_drawing(),
                2 => controller.borrow_mut().start_moving(),
                _ => {},
            }
        }

        Inhibit(false)
    }));

    drawing_area.connect_button_release_event(clone!(@strong controller, @strong header_bar => move |_dw, event| {
        if let EventType::ButtonRelease = event.get_event_type() {
            match event.get_button() {
                1 => controller.borrow_mut().end_drawing(),
                2 => controller.borrow_mut().end_moving(),
                _ => {},
            }
        }

        set_subtitle(&header_bar, controller.borrow().get_save_status());

        Inhibit(false)
    }));

    drawing_area.connect_motion_notify_event(clone!(@strong controller => move |dw, event| {
        let (x, y) = event.get_position();

        if let ShouldRedraw::Yes = controller.borrow_mut().handle_cursor(Point::new(x, y)) {
            dw.queue_draw();
        }

        Inhibit(false)
    }));

    drawing_area.connect_size_allocate(clone!(@strong controller => move |_dw, allocation| {
        controller.borrow_mut().resize(Point::new(allocation.width as f64, allocation.height as f64))
    }));

    let dwb = Rc::new(RefCell::new(drawing_area));

    // Color chooser
    let color_chooser: ColorButton = builder.get_object("color-chooser").expect("No color chooser");

    color_chooser.connect_color_set(clone!(@strong controller, @strong dwb => move |chooser| {
        controller.borrow_mut().set_color(Color::from(chooser.get_rgba()));
        dwb.borrow().queue_draw();
    }));

    // Zoom buttons
    let zoom_in_btn: Button = builder.get_object("zoom-in-btn").expect("No zoom in btn");
    zoom_in_btn.connect_clicked(clone!(@strong controller, @strong dwb => move |_btn| {
        controller.borrow_mut().zoom_in();
        dwb.borrow().queue_draw();
    }));

    let zoom_out_btn: Button = builder.get_object("zoom-out-btn").expect("No zoom out btn");
    zoom_out_btn.connect_clicked(clone!(@strong controller, @strong dwb => move |_btn| {
        controller.borrow_mut().zoom_out();
        dwb.borrow().queue_draw();
    }));

    let zoom_home_btn: Button = builder.get_object("zoom-home-btn").expect("No zoom home btn");
    zoom_home_btn.connect_clicked(clone!(@strong controller, @strong dwb => move |_btn| {
        controller.borrow_mut().go_home();
        dwb.borrow().queue_draw();
    }));

    // Undo/Redo
    let undo_menu: MenuItem = builder.get_object("undo-btn").expect("No undo btn");
    undo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb => move |_menu| {
        controller.borrow_mut().undo();
        dwb.borrow().queue_draw();
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    let redo_menu: MenuItem = builder.get_object("redo-btn").expect("No reundo btn");
    redo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb => move |_menu| {
        controller.borrow_mut().redo();
        dwb.borrow().queue_draw();
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    // File management
    let save_menu: MenuItem = builder.get_object("save-btn").expect("no save menu");
    save_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(&window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
        let res = save_file_chooser.run();

        match res {
            ResponseType::Accept => {
                if let Some(filename) = save_file_chooser.get_filename() {
                    println!("{:?}", filename);
                    controller.borrow_mut().set_saved(filename);
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                }
            },
            x => { println!("{:?}", x); },
        }
    }));

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
