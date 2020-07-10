use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::env;

use gtk::{
    Application, ApplicationWindow, DrawingArea, Builder, ColorButton,
    Button, MenuItem, FileChooserNative, FileChooserAction, ResponseType,
    HeaderBar, MessageDialog, DialogFlags, MessageType, ButtonsType, Window
};
use gdk::{EventMask, EventType, ScrollDirection};
use gtk::prelude::*;
use gio::prelude::*;
use glib::clone;
use cairo::{SvgSurface, ImageSurface, Context};

use pizarra::{
    App, app::{
        ShouldRedraw, SaveStatus, MouseButton, SelectedTool,
    }, color::Color, graphics::Transform, shape::ShapeType,
};
use pizarra::point::Point;

// Padding in pixels around the bbox of the drawing when exporting or saving
const RENDER_PADDING: f64 = 20.0;

const UNSAVED_CHANGES_SINCE_LAST_TIME: &'static str = "Hiciste algunos trazos desde la última vez\n\n¿Los quieres guardar?";
const UNSAVED_CHANGES_NEW_FILE: &'static str = "Hay algunos trazos aquí\n\n¿Los quieres guardar?";

fn set_subtitle(header_bar: &HeaderBar, save_status: &SaveStatus) {
    match save_status {
        SaveStatus::NewAndEmpty => {
            header_bar.set_title(Some("Pizarra"));
            header_bar.set_subtitle(None);
        },
        SaveStatus::NewAndChanged => {
            header_bar.set_title(Some("*Dibujo sin guardar"));
            header_bar.set_subtitle(None);
        },
        SaveStatus::Unsaved(path) => {
            header_bar.set_title(Some(&format!("*{}", path.file_name().unwrap().to_string_lossy())));
            header_bar.set_subtitle(Some(&format!("{}", path.parent().unwrap().display())));
        },
        SaveStatus::Saved(path) => {
            header_bar.set_title(Some(&format!("{}", path.file_name().unwrap().to_string_lossy())));
            header_bar.set_subtitle(Some(&format!("{}", path.parent().unwrap().display())));
        },
    }
}

fn ensure_extension(filename: &Path, extension: &str) -> PathBuf {
    if let Some(ext) = filename.extension() {
        if ext != extension {
            filename.with_extension(extension)
        } else {
            filename.into()
        }
    } else {
        filename.with_extension(extension)
    }
}

fn render_drawing(controller: &App, ctx: &Context, topleft: Point) {
    let t = Transform::new(topleft - Point::new(RENDER_PADDING, RENDER_PADDING), 1.0);
    let bgcolor = controller.bgcolor();

    ctx.set_source_rgb(bgcolor.r, bgcolor.g, bgcolor.b);
    ctx.paint();

    for cmd in controller.draw_commands_for_drawing() {
        cmd.draw(&ctx, t);
    }
}

fn save_to_svg_logic(controller: &mut App, filename: &Path) {
    if let Some([topleft, bottomright]) = controller.get_bounds() {
        let svgfilename = ensure_extension(&filename, "svg");
        let width = (bottomright.x - topleft.x).abs() + 2.0 * RENDER_PADDING;
        let height = (bottomright.y - topleft.y).abs() + 2.0 * RENDER_PADDING;
        let surface = SvgSurface::for_stream(width, height, File::create(&svgfilename).unwrap()).unwrap();
        let context = Context::new(&surface);

        render_drawing(&controller, &context, topleft);
        surface.finish_output_stream().unwrap();
        controller.set_saved(svgfilename);
    }
}

/// Implements the logic of the _save-as_ feature
fn save_as_logic<P, F>(window: &P, header_bar: &HeaderBar, controller: Rc<RefCell<App>>, callback: F)
    where
    P: IsA<Window>,
    F: Fn() -> (),
{
    let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
    let res = save_file_chooser.run();

    match res {
        ResponseType::Accept => {
            if let Some(filename) = save_file_chooser.get_filename() {
                save_to_svg_logic(&mut controller.borrow_mut(), &filename);
                set_subtitle(&header_bar, controller.borrow().get_save_status());
            }
        },
        _ => {},
    }

    callback();
}

fn open_logic<P>(window: &P, header_bar: &HeaderBar, controller: Rc<RefCell<App>>)
    where P: IsA<Window>
{
    let open_file_chooser = FileChooserNative::new(Some("Abrir"), Some(window), FileChooserAction::Open, Some("Abrir"), Some("Cancelar"));
    let res = open_file_chooser.run();

    match res {
        ResponseType::Accept => {
            if let Some(filename) = open_file_chooser.get_filename() {
                if let Ok(mut file) = File::open(&filename) {
                    let mut svg = String::new();

                    if let Ok(_) = file.read_to_string(&mut svg) {
                        if let Ok(_) = controller.borrow_mut().open(&svg) {
                            controller.borrow_mut().set_saved(filename);
                            set_subtitle(&header_bar, controller.borrow().get_save_status());
                        } else {
                            // failed parsing
                        }
                    } else {
                        // read failed
                    }
                } else {
                    // open failed
                }
            }
        },
        _ => {},
    }
}

/// Implements the logic of the export feature
fn export_logic<P: IsA<Window>>(window: &P, controller: Rc<RefCell<App>>) {
    let export_file_chooser = FileChooserNative::new(Some("Exportar"), Some(window), FileChooserAction::Save, Some("Exportar"), Some("Cancelar"));
    let res = export_file_chooser.run();

    match res {
        ResponseType::Accept => {
            if let Some(filename) = export_file_chooser.get_filename() {
                if let Some([topleft, bottomright]) = controller.borrow().get_bounds() {
                    let pngfilename = ensure_extension(&filename, "png");
                    let width = (bottomright.x - topleft.x).abs() + 2.0 * RENDER_PADDING;
                    let height = (bottomright.y - topleft.y).abs() + 2.0 * RENDER_PADDING;
                    let surface = ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32).unwrap();
                    let context = cairo::Context::new(&surface);

                    render_drawing(&controller.borrow(), &context, topleft);

                    surface.write_to_png(&mut File::create(pngfilename).unwrap()).unwrap();
                }
            }
        },
        _ => {},
    }
}

fn yes_no_cancel_dialog<F, G, H>(window: &ApplicationWindow, message: &str, yes_callback: F, no_callback: G, cancel_callback: H) -> Inhibit
    where
    F: Fn() -> Inhibit,
    G: Fn() -> Inhibit,
    H: Fn() -> Inhibit,
{
    let message_dialog = MessageDialog::new(
        Some(window),
        DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Question,
        ButtonsType::None,
        message,
    );
    message_dialog.add_button("Sí", ResponseType::Yes);
    message_dialog.add_button("No", ResponseType::No);
    message_dialog.add_button("Cancelar", ResponseType::Cancel);

    let response = message_dialog.run();

    message_dialog.hide();

    match response {
        ResponseType::Yes => yes_callback(),
        ResponseType::Cancel | ResponseType::DeleteEvent => cancel_callback(),
        _ => no_callback(),
    }
}

fn init(app: &Application, filename: Option<PathBuf>) {
    // Initialize layout from .glade file
    let layout = include_str!("../res/layout.glade");
    let builder = Builder::new_from_string(layout);
    let controller = Rc::new(RefCell::new(App::new(Point::new(1.0, 1.0))));
    let window: ApplicationWindow = builder.get_object("main-window").expect("Couldn't get window");
    let header_bar: HeaderBar = builder.get_object("header-bar").expect("no header bar");

    window.set_application(Some(app));

    if let Some(filename) = filename {
        let mut svg = String::new();
        let mut file = File::open(&filename).expect("Could not open given file");

        file.read_to_string(&mut svg).expect("could not read file contents");
        controller.borrow_mut().open(&svg).expect("Could not parse given file");
        controller.borrow_mut().set_saved(filename);
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }

    // save on exit
    window.connect_delete_event(clone!(@strong controller, @strong window => move |_window, _event| {
        match controller.borrow().get_save_status() {
            SaveStatus::NewAndChanged => yes_no_cancel_dialog(&window, "Ya hiciste algunos dibujos\n\n¿Deseas guardar?", || {
                let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(&window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
                let res = save_file_chooser.run();

                match res {
                    ResponseType::Accept => {
                        if let Some(filename) = save_file_chooser.get_filename() {
                            save_to_svg_logic(&mut controller.borrow_mut(), &filename);
                        }
                    },
                    _ => {},
                }

                Inhibit(false)
            }, || {
                Inhibit(false)
            }, || {
                Inhibit(true)
            }),
            SaveStatus::NewAndEmpty => Inhibit(false),
            SaveStatus::Saved(_path) => Inhibit(false),
            SaveStatus::Unsaved(path) => yes_no_cancel_dialog(&window, "Hay cambios desde la última vez que guardaste\n\n¿Quieres guardarlos?", || {
                    save_to_svg_logic(&mut controller.borrow_mut(), &path);
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                }, || {
                    Inhibit(true)
                }),
        }
    }));

    // Drawing area
    let drawing_area: DrawingArea = builder.get_object("drawing-area").expect("No drawing_area");

    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK
        | EventMask::SCROLL_MASK;

    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    drawing_area.connect_draw(clone!(@strong controller => move |_dw, ctx| {
        let controller = controller.borrow();
        let t = controller.get_transform();
        let bgcolor = Color::black();

        let commands = controller.draw_commands_for_screen();

        ctx.set_source_rgb(bgcolor.r, bgcolor.g, bgcolor.b);
        ctx.paint();

        // content
        for cmd in commands {
            cmd.draw(ctx, t);
        }

        Inhibit(false)
    }));

    drawing_area.connect_key_release_event(|_dw, _event| {
        Inhibit(false)
    });

    drawing_area.connect_scroll_event(clone!(@strong controller => move |dw, event| {
        if let Some(direction) = event.get_scroll_direction() {
            match direction {
                ScrollDirection::Up => {
                    controller.borrow_mut().zoom_in();
                    dw.queue_draw();
                },
                ScrollDirection::Down => {
                    controller.borrow_mut().zoom_out();
                    dw.queue_draw();
                },
                _ => {},
            }
        }

        Inhibit(false)
    }));

    drawing_area.connect_button_press_event(clone!(@strong controller => move |_dw, event| {
        if let EventType::ButtonPress = event.get_event_type() {
            controller.borrow_mut().handle_mouse_button_pressed(match event.get_button() {
                1 => MouseButton::Left,
                2 => MouseButton::Middle,
                3 => MouseButton::Right,
                _ => MouseButton::Unknown,
            }, Point::from(event.get_position()));
        }

        Inhibit(false)
    }));

    drawing_area.connect_button_release_event(clone!(@strong controller, @strong header_bar => move |_dw, event| {
        if let EventType::ButtonRelease = event.get_event_type() {
            controller.borrow_mut().handle_mouse_button_released(match event.get_button() {
                1 => MouseButton::Left,
                2 => MouseButton::Middle,
                3 => MouseButton::Right,
                _ => MouseButton::Unknown,
            }, Point::from(event.get_position()));
        }

        set_subtitle(&header_bar, controller.borrow().get_save_status());

        Inhibit(false)
    }));

    drawing_area.connect_motion_notify_event(clone!(@strong controller => move |dw, event| {
        let (x, y) = event.get_position();

        if let ShouldRedraw::Yes = controller.borrow_mut().handle_mouse_move(Point::new(x, y)) {
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
    let open_menu: MenuItem = builder.get_object("open-btn").expect("no open menu");
    open_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => open_logic(&window, &header_bar, controller.clone()),
            SaveStatus::NewAndChanged => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_NEW_FILE, clone!(@strong controller, @strong header_bar, @strong window => move || {
                    save_as_logic(&window, &header_bar, controller.clone(), clone!(@strong controller, @strong header_bar, @strong window => move || {
                        open_logic(&window, &header_bar, controller.clone());
                    }));
                    Inhibit(false)
                }), clone!(@strong controller, @strong header_bar, @strong window => move || {
                    open_logic(&window, &header_bar, controller.clone());
                    Inhibit(false)
                }), || {
                    Inhibit(false)
                });
            },
            SaveStatus::Unsaved(path) => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_SINCE_LAST_TIME, clone!(@strong controller, @strong header_bar, @strong window => move || {
                    save_to_svg_logic(&mut controller.borrow_mut(), &path);
                    open_logic(&window, &header_bar, controller.clone());
                    Inhibit(false)
                }), || {
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Saved(_path) => open_logic(&window, &header_bar, controller.clone()),
        }
    }));

    let new_menu: MenuItem = builder.get_object("new-btn").expect("no new menu");
    new_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window, @strong dwb => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {},
            SaveStatus::NewAndChanged => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_NEW_FILE, || {
                    save_as_logic(&window, &header_bar, controller.clone(), || {
                    });
                    Inhibit(false)
                }, || {
                    controller.borrow_mut().reset();
                    dwb.borrow().queue_draw();
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Unsaved(path) => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_SINCE_LAST_TIME, || {
                    save_to_svg_logic(&mut controller.borrow_mut(), &path);
                    controller.borrow_mut().reset();
                    dwb.borrow().queue_draw();
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
                }, || {
                    controller.borrow_mut().reset();
                    dwb.borrow().queue_draw();
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Saved(_path) => {
                controller.borrow_mut().reset();
                dwb.borrow().queue_draw();
                set_subtitle(&header_bar, controller.borrow().get_save_status());
            },
        }
    }));

    let save_menu: MenuItem = builder.get_object("save-btn").expect("no save menu");
    save_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {}, // nothing to save actually
            SaveStatus::NewAndChanged => {
                save_as_logic(&window, &header_bar, controller.clone(), || {
                });
            },
            SaveStatus::Unsaved(path) => {
                save_to_svg_logic(&mut controller.borrow_mut(), &path);
                controller.borrow_mut().set_saved(path.clone());
                set_subtitle(&header_bar, controller.borrow().get_save_status());
            },
            SaveStatus::Saved(_path) => {},
        }
    }));

    let save_as_menu: MenuItem = builder.get_object("save-as-btn").expect("no save as menu");
    save_as_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {},
            SaveStatus::NewAndChanged => {
                save_as_logic(&window, &header_bar, controller.clone(), || {
                });
            },
            SaveStatus::Unsaved(_path) => {
                save_as_logic(&window, &header_bar, controller.clone(), || {
                });
            },
            SaveStatus::Saved(_path) => {
                save_as_logic(&window, &header_bar, controller.clone(), || {
                });
            },
        }
    }));

    let export_menu: MenuItem = builder.get_object("export-btn").expect("no export menu");
    export_menu.connect_activate(clone!(@strong controller, @strong window => move |_menu| {
        export_logic(&window, controller.clone());
    }));

    // Change shape
    let set_pen_menu: MenuItem = builder.get_object("tool-pen-btn").expect("no pen menu");
    set_pen_menu.connect_activate(clone!(@strong controller => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeType::Line));
    }));

    let set_rectangle_menu: MenuItem = builder.get_object("tool-rect-btn").expect("no pen menu");
    set_rectangle_menu.connect_activate(clone!(@strong controller => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeType::Rectangle));
    }));

    let set_eraser_menu: MenuItem = builder.get_object("tool-eraser-btn").expect("no pen menu");
    set_eraser_menu.connect_activate(clone!(@strong controller => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Eraser);
    }));

    // Show
    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("tk.categulario.pizarra"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    let arguments: Vec<_> = env::args().collect();

    application.connect_activate(move |app| init(app, arguments.get(1).map(|f| f.into())));

    application.run(&[]);
}
