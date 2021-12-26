use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::path_command::{PathCommand, CubicBezierCurve};
use crate::shape::{ShapeBuilder, ShapeFinished, path::Path};
use crate::draw_commands::DrawCommand;
use crate::transform::Transform;

/// Enum that handles the states of building a bezier-polygon
#[derive(Debug, Copy, Clone)]
enum BuilderState {
    /// Initial state, only on initialization
    Init(Vec2DWorld),

    /// Transition to this state on pressed and change to something else
    /// immediatly after
    PointStart(Vec2DWorld),

    /// Transition to this from PointStart if the next event is mouse move
    Handle {
        to: Vec2DWorld,
        pt1: Vec2DWorld,
    },

    /// Transition to this after a release event
    FreePoint(Vec2DWorld),

    /// End is detected on the pressed event but this event doesn't return a
    /// finished shape, so this state is transitioned to on pressed and then
    /// detected on relesease to return the built shape
    End,
}

#[derive(Debug)]
pub struct PolygonBuilder {
    points: Vec<PathCommand<Vec2DWorld>>,
    tip: BuilderState,
    thickness: f64,
    color: Color,
}

impl PolygonBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> PolygonBuilder {
        let mut init_vec = Vec::with_capacity(8);

        init_vec.push(PathCommand::MoveTo(initial));

        PolygonBuilder {
            points: init_vec,
            tip: BuilderState::Init(initial),
            thickness,
            color,
        }
    }

    fn touches_prev_point(&self, point: Vec2DScreen, t: Transform, radius: f64) -> Option<Vec2DWorld> {
        self.points.iter().map(|p| p.to()).find(|&p| {
            t.to_screen_coordinates(p).distance(point) <= radius
        })
    }
}

impl ShapeBuilder for PolygonBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        let pos = t.to_world_coordinates(pos);

        match self.tip {
            BuilderState::Init(p) => {
                self.points.push(PathCommand::MoveTo(p));

                self.tip = BuilderState::Handle { to: pos, pt1: pos };
            }
            BuilderState::PointStart(p) => {
                self.tip = BuilderState::Handle { to: p, pt1: pos };
            }
            BuilderState::FreePoint(_) => {
                self.tip = BuilderState::FreePoint(pos);
            }
            BuilderState::Handle { .. } => {
                self.tip = BuilderState::Handle { to: pos, pt1: pos };
            }
            BuilderState::End => {}
        }
    }

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) {
        match self.tip {
            BuilderState::FreePoint(p) => {
                if let Some(point) = self.touches_prev_point(pos, t, snap) {
                    self.points.push(PathCommand::LineTo(point));
                    self.tip = BuilderState::End;
                } else {
                    self.tip = BuilderState::PointStart(p);
                }
            }
            _ => {}
        }
    }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) -> ShapeFinished {
        let pos = t.to_world_coordinates(pos);

        match self.tip {
            BuilderState::Init(p) => {
                self.points.push(PathCommand::MoveTo(p));
                self.tip = BuilderState::FreePoint(pos);

                ShapeFinished::No
            }
            BuilderState::PointStart(p) => {
                self.points.push(PathCommand::LineTo(p));
                self.tip = BuilderState::FreePoint(pos);

                ShapeFinished::No
            }
            BuilderState::Handle { to, pt1 } => {
                self.points.push(PathCommand::CurveTo(CubicBezierCurve {
                    pt1,
                    pt2: to,
                    to,
                }));

                self.tip = BuilderState::FreePoint(pos);

                ShapeFinished::No
            }
            BuilderState::FreePoint(_) => ShapeFinished::No,
            BuilderState::End => {
                ShapeFinished::Yes(Box::new(Path::from_parts(
                    self.points.clone(),
                    Some(self.color),
                    None,
                    self.thickness,
                )))
            }
        }
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        let mut path_commands = self.points.clone();

        match self.tip {
            BuilderState::Init(p) => {
                path_commands.push(PathCommand::MoveTo(p));
            }
            BuilderState::Handle { to: point, pt1: handle } => {
                path_commands.push(PathCommand::CurveTo(CubicBezierCurve {
                    pt1: handle,
                    pt2: point,
                    to: point,
                }));
            }
            BuilderState::FreePoint(p) => {
                path_commands.push(PathCommand::LineTo(p));
            }
            BuilderState::PointStart(p) => {
                path_commands.push(PathCommand::LineTo(p));
            }
            BuilderState::End => {}
        }

        let mut commands = Vec::with_capacity(self.points.len() * 2);

        commands.push(DrawCommand::Path {
            commands: path_commands,
            thickness: self.thickness,
            color: Some(self.color),
            fill: None,
        });

        commands.extend(self.points.iter().map(|p| {
            match self.tip {
                BuilderState::FreePoint(pf) if pf.distance(p.to()) <= snap => {
                    DrawCommand::ScreenCircle {
                        center: t.to_screen_coordinates(p.to()),
                        radius: snap,
                        color: None,
                        fill: Some(Color::green().half_transparent()),
                        thickness: 1.0,
                    }
                }
                _ => DrawCommand::ScreenCircle {
                    center: t.to_screen_coordinates(p.to()),
                    radius: snap,
                    color: None,
                    fill: Some(Color::gray().half_transparent()),
                    thickness: 1.0,
                }
            }
        }));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAP: f64 = 10.0;

    /// This tests the polygon's normal behavior of clicking here and there
    /// until some last click falls very close (in screen units) from one of the
    /// previous ones
    #[test]
    fn it_behaves() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(20.0, 0.0), t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 0.0), t, SNAP) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 20.0), t, SNAP) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP) {
            panic!()
        }
    }

    /// To close a line you have to click somewhere near a previous point. This
    /// test checks that the end point of a polygon matches exactly the point
    /// the user was aiming to even if the event is registered somewhere else.
    #[test]
    fn no_clunky_path_ends() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(30.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(30.0, 0.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(30.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(29.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(29.0, 0.0), t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(29.0, 0.0), t, SNAP) {
            panic!()
        }

        assert_eq!(poly.draw_commands(t, SNAP)[0], DrawCommand::Path {
            color: Some(Default::default()),
            fill: None,
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2DWorld::new(30.0, 0.0)),
                PathCommand::LineTo(Vec2DWorld::new(30.0, 0.0)),
            ],
            thickness: 4.0,
        });
    }

    #[test]
    fn can_render_first_point_when_building() {
        let t = Default::default();
        let poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 3.0);

        assert_eq!(poly.draw_commands(t, SNAP)[0], DrawCommand::Path {
            color: Some(Default::default()),
            fill: None,
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
            ],
            thickness: 3.0,
        });
    }

    fn make_this_poly(t: Transform) -> PolygonBuilder {
        let center = Vec2DWorld::new(0.0, 0.0);
        let pf = Vec2DWorld::new(0.0, -100.0);
        let mut poly = PolygonBuilder::start(Default::default(), center, 4.0);

        poly.handle_button_released(t.to_screen_coordinates(center), t, SNAP);
        poly.handle_mouse_moved(t.to_screen_coordinates(pf), t, SNAP);
        poly.handle_button_pressed(t.to_screen_coordinates(pf), t, SNAP);
        poly.handle_button_released(t.to_screen_coordinates(pf), t, SNAP);

        poly
    }

    #[test]
    fn the_polygon_closes_itself_in_screen_units() {
        // at home scale
        let screen = Vec2DScreen::new(800.0, 600.0);
        let center = screen / 2.0;

        // No matter the zoom level the first point should finish a shape but
        // the second shouldn't at the default 10 px radius
        let p1 = center + Vec2DScreen::new(9.9, 0.0);
        let p2 = center + Vec2DScreen::new(10.1, 0.0);

        let t0 = Transform::default_for_viewport(screen);

        // closes polygon at initial zoom
        let mut poly = make_this_poly(t0);
        let t = t0;
        poly.handle_mouse_moved(p1, t, SNAP);
        poly.handle_button_pressed(p1, t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        let mut poly = make_this_poly(t0);
        poly.handle_mouse_moved(p2, t, SNAP);
        poly.handle_button_pressed(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }

        // at zoomed in scale closes at shorter distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(2.0, center);
        poly.handle_mouse_moved(p1, t, SNAP);
        poly.handle_button_pressed(p1, t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        let mut poly = make_this_poly(t0);
        poly.handle_mouse_moved(p2, t, SNAP);
        poly.handle_button_pressed(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }

        // at zoomed out scale closes at greater distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(0.5, center);
        poly.handle_mouse_moved(p1, t, SNAP);
        poly.handle_button_pressed(p1, t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        let mut poly = make_this_poly(t0);
        poly.handle_mouse_moved(p2, t, SNAP);
        poly.handle_button_pressed(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }
    }

    /// for some extra visual aid if the cursor is within one of the
    #[test]
    fn guide_circles_color_when_cursor_is_close() {
        let t: Transform = Default::default();
        let mut poly = PolygonBuilder::start(Color::green(), Vec2DWorld::new(0.0, 0.0), 2.0);

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);
        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        let commands = poly.draw_commands(t, SNAP);

        assert_eq!(&commands[1], &DrawCommand::ScreenCircle {
            center: Vec2DScreen::new(0.0, 0.0),
            radius: SNAP,
            color: None,
            fill: Some(Color::green().half_transparent()),
            thickness: 1.0,
        });
    }

    /// test that a shape with smooth border can be made. It will look like a
    /// potatoe
    #[test]
    fn the_all_mighty_bezier_polygon() {
        let t: Transform = Default::default();

        // the points
        let a = Vec2DScreen::new(0.0, -80.0);
        let b = Vec2DScreen::new(30.0, -80.0);
        let c = Vec2DScreen::new(80.0, 0.0);
        let d = Vec2DScreen::new(80.0, 20.0);
        let e = Vec2DScreen::new(0.0, 60.0);
        let f = Vec2DScreen::new(-20.0, 60.0);
        let g = Vec2DScreen::new(-80.0, 0.0);
        let h = Vec2DScreen::new(-80.0, -20.0);

        let mut poly = PolygonBuilder::start(Color::green(), t.to_world_coordinates(a), 2.0);

        poly.handle_mouse_moved(b, t, SNAP);
        poly.handle_button_released(b, t, SNAP);

        poly.handle_mouse_moved(c, t, SNAP);
        poly.handle_button_pressed(c, t, SNAP);
        poly.handle_mouse_moved(d, t, SNAP);
        poly.handle_button_released(d, t, SNAP);

        poly.handle_mouse_moved(e, t, SNAP);
        poly.handle_button_pressed(e, t, SNAP);
        poly.handle_mouse_moved(f, t, SNAP);
        poly.handle_button_released(f, t, SNAP);

        poly.handle_mouse_moved(g, t, SNAP);
        poly.handle_button_pressed(g, t, SNAP);
        poly.handle_mouse_moved(h, t, SNAP);
        poly.handle_button_released(h, t, SNAP);

        poly.handle_mouse_moved(a, t, SNAP);
        poly.handle_button_pressed(a, t, SNAP);
        if let ShapeFinished::Yes(s) = poly.handle_button_released(a, t, SNAP) {
            assert_eq!(s.draw_commands(), DrawCommand::Path {
                commands: vec![
                    PathCommand::MoveTo(t.to_world_coordinates(a)),
                    PathCommand::CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(b),
                        pt2: Vec2DWorld::new(80.0, -20.0),
                        to: t.to_world_coordinates(c),
                    }),
                    PathCommand::CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(d),
                        pt2: Vec2DWorld::new(20.0, 60.0),
                        to: t.to_world_coordinates(e),
                    }),
                    PathCommand::CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(f),
                        pt2: Vec2DWorld::new(-80.0, 20.0),
                        to: t.to_world_coordinates(g),
                    }),
                    PathCommand::CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(h),
                        pt2: Vec2DWorld::new(-30.0, -80.0),
                        to: t.to_world_coordinates(a),
                    }),
                ],
                color: Some(Color::green()),
                fill: None,
                thickness: 2.0,
            });
        } else {
            panic!();
        }
    }

    /// at the moment of writing dragging the first point just creates half a
    /// line, but it should create a handle that makes a smooth start although
    /// no line is rendered until mouse is released
    #[test]
    fn dragging_first_point_just_creates_a_handle() {
        todo!()
    }

    /// When creating a polygon with a stylus it is possible that there are a
    /// few 'move' events between a button press and a button release. That
    /// would make creating sharp polygons impossible. What should happen
    /// instead is that if a release event is too close to its matching press
    /// event it should fix the shape to a sharp angle.
    #[test]
    fn sharp_polygons_are_still_possible_by_hand() {
        todo!()
    }
}
