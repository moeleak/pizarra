use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::path_command::{PathCommand::{self, *}, CubicBezierCurve};
use crate::shape::{ShapeBuilder, ShapeFinished, path::Path};
use crate::draw_commands::DrawCommand;
use crate::transform::Transform;
use crate::geom::mirror;

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
    FreePoint {
        pt1: Vec2DWorld,
        pt2: Vec2DWorld,
        to: Vec2DWorld,
    },

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

        init_vec.push(MoveTo(initial));

        PolygonBuilder {
            points: init_vec,
            tip: BuilderState::Init(initial),
            thickness,
            color,
        }
    }

    /// Tells if the given point is near a previous point of the polygon.
    ///
    /// The mesure for near is being closer than `radius` in screen units. The
    /// return value can be one of the following things:
    ///
    /// 1. Some(Some((point, handle)))
    ///    This means that the last section of the polygon must be a curve to
    ///    the returned point using the given handle
    /// 2. Some(None)
    ///    The nearest point of the polygon to the given point is the last one.
    ///    It is not necessary to add more sections
    /// 3. None
    ///    It is not close enough to any of the polygon's points
    fn touches_prev_point(&self, point: Vec2DWorld, t: Transform, radius: f64) -> Option<Option<(Vec2DWorld, Vec2DWorld)>> {
        let point = t.to_screen_coordinates(point);

        // Check all but the last point for a coincidence
        let ans = self.points.iter().zip(self.points.iter().skip(1)).find(|&(&command, _)| {
            t.to_screen_coordinates(command.to()).distance(point) <= radius
        }).map(|(&a, &b)| {
            if let MoveTo(p) = a {
                if let CurveTo(c) = b {
                    (p, mirror(c.pt1, p))
                } else {
                    (p, p)
                }
            } else {
                (a.to(), a.to())
            }
        });

        if let Some(ans) = ans {
            return Some(Some(ans));
        }

        self.points.last().map(|last| {
            if t.to_screen_coordinates(last.to()).distance(point) <= radius {
                Some(None)
            } else {
                None
            }
        }).flatten()
    }
}

impl ShapeBuilder for PolygonBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        let pos = t.to_world_coordinates(pos);

        match self.tip {
            BuilderState::Init(_) => {
                self.tip = BuilderState::Handle { pt1: pos, to: pos };
            }
            BuilderState::PointStart(p) | BuilderState::Handle { to: p, .. } => {
                self.tip = BuilderState::Handle { to: p, pt1: pos };
                if let Some(last) = self.points.last_mut() {
                    if let CurveTo(c) = last {
                        *last = CurveTo(CubicBezierCurve {
                            pt2: mirror(pos, p),
                            ..*c
                        });
                    }
                }
            }
            BuilderState::FreePoint { pt1, .. } => {
                self.tip = BuilderState::FreePoint { pt1, pt2: pos, to: pos };
            }
            BuilderState::End => {}
        }
    }

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) {
        let pos = t.to_world_coordinates(pos);

        if let BuilderState::FreePoint { pt1, .. } = self.tip {
            match self.touches_prev_point(pos, t, snap) {
                // match with first point or middle. Join it smoothly if
                // first point is a handle
                Some(Some((point, pt2))) => {
                    self.points.push(CurveTo(CubicBezierCurve {
                        pt1, pt2, to: point,
                    }));
                    self.tip = BuilderState::End;
                }
                // match with end point. Do not add a new segment to the
                // path
                Some(None) => {
                    self.tip = BuilderState::End;
                }
                // No match, just transition to a free point
                None => {
                    self.points.push(CurveTo(CubicBezierCurve {
                        pt1, pt2: pos, to: pos,
                    }));
                    self.tip = BuilderState::PointStart(pos);
                }
            }
        }
    }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        let pos = t.to_world_coordinates(pos);

        match self.tip {
            BuilderState::Init(p) => {
                self.tip = BuilderState::FreePoint { pt1: p, pt2: pos, to: pos };

                ShapeFinished::No
            }
            BuilderState::PointStart(p) => {
                self.tip = BuilderState::FreePoint { pt1: p, pt2: pos, to: pos };

                ShapeFinished::No
            }
            BuilderState::Handle { pt1, .. } => {
                self.tip = if let Some(last) = self.points.last_mut() {
                    if t.to_screen_coordinates(last.to()).distance(t.to_screen_coordinates(pt1)) < snap {
                        // the handle didn't move more than _snap, probably a
                        // result of the sensitivity of the pen. The line will
                        // be fixed to look perfectly straight.

                        if let CurveTo(c) = last {
                            c.pt2 = c.to;
                        }

                        BuilderState::FreePoint {
                            pt1: last.to(),
                            pt2: pos,
                            to: pos,
                        }
                    } else {
                        BuilderState::FreePoint { pt1, pt2: pos, to: pos }
                    }
                } else {
                    BuilderState::FreePoint { pt1, pt2: pos, to: pos }
                };

                ShapeFinished::No
            }
            BuilderState::FreePoint { .. } => ShapeFinished::No,
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

        // add a command to the path_commands depending on the last part of the
        // polygon. This is what the user is manipulating and changes on every
        // new input.
        match self.tip {
            BuilderState::Init(_) | BuilderState::End => {}
            BuilderState::Handle { pt1, .. } => {
                path_commands.push(CurveTo(CubicBezierCurve {
                    pt1,
                    pt2: pt1,
                    to: pt1,
                }));
            }
            BuilderState::FreePoint { pt1, pt2, to } => {
                if let Some(Some((p, h))) = self.touches_prev_point(to, t, snap) {
                    path_commands.push(CurveTo(CubicBezierCurve {
                        pt1, pt2: h, to: p,
                    }));
                } else {
                    path_commands.push(CurveTo(CubicBezierCurve {
                        pt1, pt2, to,
                    }));
                }
            }
            BuilderState::PointStart(p) => {
                path_commands.push(LineTo(p));
            }
        }

        // this vector holds the commands that will be returned. Not only the
        // polygon itself but the helper circles and lines
        let mut commands = Vec::with_capacity(self.points.len() * 2);

        // add the polygon
        commands.push(DrawCommand::Path {
            commands: path_commands.clone(),
            thickness: self.thickness,
            color: Some(self.color),
            fill: None,
        });

        // for every point of the polygon add a helper circle and bezier handler
        // lines
        commands.extend(path_commands.iter().zip(path_commands.iter().skip(1)).enumerate().map(|(i, (p, next))| {
            let mut commands = Vec::with_capacity(3);

            // if the first command (except for the MoveTo) is a CurveTo then
            // draw a handle in the oposite direction to suggest the user that
            // the closing of a bezier polygon is soft
            if i == 0 {
                if let CurveTo(c) = next {
                    commands.push(DrawCommand::Path {
                        commands: vec![
                            MoveTo(p.to()),
                            LineTo(mirror(c.pt1, p.to())),
                        ],
                        color: Some(Color::gray()),
                        fill: None,
                        thickness: 1.0,
                    });
                }
            }

            // the bezier handles
            if let CurveTo(c) = next {
                commands.push(DrawCommand::Path {
                    commands: vec![
                        MoveTo(c.to),
                        LineTo(c.pt2),
                    ],
                    color: Some(Color::gray()),
                    fill: None,
                    thickness: 1.0,
                });
                commands.push(DrawCommand::Path {
                    commands: vec![
                        MoveTo(p.to()),
                        LineTo(c.pt1),
                    ],
                    color: Some(Color::gray()),
                    fill: None,
                    thickness: 1.0,
                });
            }

            // the circles
            commands.push(DrawCommand::ScreenCircle {
                center: t.to_screen_coordinates(p.to()),
                radius: snap,
                color: None,
                fill: Some(match self.tip {
                    BuilderState::FreePoint { to, .. } if to.distance(p.to()) <= snap => {
                        Color::green().half_transparent()
                    }
                    _ => Color::gray().half_transparent()
                }),
                thickness: 1.0,
            });

            commands
        }).flatten());

        commands
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(20.0, 20.0), t, SNAP) {
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
                MoveTo(Vec2DWorld::new(0.0, 0.0)),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2DWorld::new(0.0, 0.0),
                    pt2: Vec2DWorld::new(30.0, 0.0),
                    to: Vec2DWorld::new(30.0, 0.0),
                }),
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
                MoveTo(Vec2DWorld::new(0.0, 0.0)),
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

        assert_eq!(commands.last().unwrap(), &DrawCommand::ScreenCircle {
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
                    MoveTo(t.to_world_coordinates(a)),
                    CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(b),
                        pt2: Vec2DWorld::new(80.0, -20.0),
                        to: t.to_world_coordinates(c),
                    }),
                    CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(d),
                        pt2: Vec2DWorld::new(20.0, 60.0),
                        to: t.to_world_coordinates(e),
                    }),
                    CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(f),
                        pt2: Vec2DWorld::new(-80.0, 20.0),
                        to: t.to_world_coordinates(g),
                    }),
                    CurveTo(CubicBezierCurve {
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

    /// When creating a polygon with a stylus it is possible that there are a
    /// few 'move' events between a button press and a button release. That
    /// would make creating sharp polygons impossible. What should happen
    /// instead is that if a release event is too close to its matching press
    /// event it should fix the shape to a sharp angle.
    #[test]
    fn sharp_polygons_are_still_possible_by_hand() {
        let t: Transform = Default::default();
        let mut poly = PolygonBuilder::start(Color::green(), Vec2DWorld::new(0.0, 0.0), 2.0);

        // move slightly after starting the polygon
        poly.handle_mouse_moved(Vec2DScreen::new(1.0, 1.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(1.0, 1.0), t, SNAP);

        // second point, also drag a little bit
        poly.handle_mouse_moved(Vec2DScreen::new(0.0, -50.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(0.0, -50.0), t, SNAP);
        poly.handle_mouse_moved(Vec2DScreen::new(1.0, -51.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(1.0, -51.0), t, SNAP);

        // third point
        poly.handle_mouse_moved(Vec2DScreen::new(50.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(50.0, 0.0), t, SNAP);
        poly.handle_mouse_moved(Vec2DScreen::new(51.0, 1.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(51.0, 1.0), t, SNAP);

        // closing
        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t, SNAP);
        poly.handle_button_pressed(Vec2DScreen::new(0.0, 0.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        let commands = poly.draw_commands(t, SNAP);

        assert_eq!(commands, vec![
            DrawCommand::Path {
                color: Some(Color { r: 138, g: 226, b: 52, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: 0.0 }),
                    CurveTo(CubicBezierCurve { pt1: Vec2DWorld { x: 0.0, y: 0.0 }, pt2: Vec2DWorld { x: 0.0, y: -50.0 }, to: Vec2DWorld { x: 0.0, y: -50.0 } }),
                    CurveTo(CubicBezierCurve { pt1: Vec2DWorld { x: 0.0, y: -50.0 }, pt2: Vec2DWorld { x: 50.0, y: 0.0 }, to: Vec2DWorld { x: 50.0, y: 0.0 } }),
                    CurveTo(CubicBezierCurve { pt1: Vec2DWorld { x: 50.0, y: 0.0 }, pt2: Vec2DWorld { x: 0.0, y: 0.0 }, to: Vec2DWorld { x: 0.0, y: 0.0 } })
                ], thickness: 2.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: 0.0 }), LineTo(Vec2DWorld { x: 0.0, y: 0.0 })], thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: -50.0 }), LineTo(Vec2DWorld { x: 0.0, y: -50.0 })], thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: 0.0 }), LineTo(Vec2DWorld { x: 0.0, y: 0.0 })], thickness: 1.0 },
            DrawCommand::ScreenCircle {
                center: Vec2DScreen { x: 0.0, y: 0.0 }, radius: 10.0, color: None,
                fill: Some(Color { r: 148, g: 148, b: 148, a: 127 }), thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 50.0, y: 0.0 }), LineTo(Vec2DWorld { x: 50.0, y: 0.0 })], thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: -50.0 }), LineTo(Vec2DWorld { x: 0.0, y: -50.0 })], thickness: 1.0 },
            DrawCommand::ScreenCircle {
                center: Vec2DScreen { x: 0.0, y: -50.0 }, radius: 10.0, color: None,
                fill: Some(Color { r: 148, g: 148, b: 148, a: 127 }), thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 0.0, y: 0.0 }), LineTo(Vec2DWorld { x: 0.0, y: 0.0 })], thickness: 1.0 },
            DrawCommand::Path {
                color: Some(Color { r: 148, g: 148, b: 148, a: 255 }),
                fill: None, commands: vec![
                    MoveTo(Vec2DWorld { x: 50.0, y: 0.0 }), LineTo(Vec2DWorld { x: 50.0, y: 0.0 })], thickness: 1.0 },
            DrawCommand::ScreenCircle {
                center: Vec2DScreen { x: 50.0, y: 0.0 }, radius: 10.0, color: None,
                fill: Some(Color { r: 148, g: 148, b: 148, a: 127 }), thickness: 1.0 }
        ]);
    }

    /// when dragging the handle for a point it should paint the handle lines
    /// instead of an incomplete path
    #[test]
    fn handles_are_rendered_instead_of_incomplete_paths() {
        let t: Transform = Default::default();

        // the points
        let a = Vec2DScreen::new(0.0, -80.0);
        let b = Vec2DScreen::new(30.0, -80.0);
        let c = Vec2DScreen::new(80.0, 0.0);
        let d = Vec2DScreen::new(80.0, 20.0);

        let mut poly = PolygonBuilder::start(Color::green(), t.to_world_coordinates(a), 2.0);

        poly.handle_mouse_moved(b, t, SNAP);
        poly.handle_button_released(b, t, SNAP);

        poly.handle_mouse_moved(c, t, SNAP);
        poly.handle_button_pressed(c, t, SNAP);
        poly.handle_mouse_moved(d, t, SNAP);

        assert_eq!(poly.draw_commands(t, SNAP), vec![
            // the paths
            DrawCommand::Path {
                commands: vec![
                    MoveTo(t.to_world_coordinates(a)),
                    CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(b),
                        pt2: Vec2DWorld::new(80.0, -20.0),
                        to: t.to_world_coordinates(c),
                    }),
                    CurveTo(CubicBezierCurve {
                        pt1: t.to_world_coordinates(d),
                        pt2: t.to_world_coordinates(d),
                        to: t.to_world_coordinates(d),
                    }),
                ],
                color: Some(Color::green()),
                fill: None,
                thickness: 2.0,
            },

            // the helpers
            DrawCommand::Path { color: Some(Color::gray()), fill: None, commands: vec![
                MoveTo(Vec2DWorld { x: 0.0, y: -80.0 }), LineTo(Vec2DWorld { x: -30.0, y: -80.0 })
            ], thickness: 1.0 },
            DrawCommand::Path { color: Some(Color::gray()), fill: None, commands: vec![
                MoveTo(Vec2DWorld { x: 80.0, y: 0.0 }), LineTo(Vec2DWorld { x: 80.0, y: -20.0 })
            ], thickness: 1.0 },
            DrawCommand::Path { color: Some(Color::gray()), fill: None, commands: vec![
                MoveTo(Vec2DWorld { x: 0.0, y: -80.0 }), LineTo(Vec2DWorld { x: 30.0, y: -80.0 })
            ], thickness: 1.0 },
            DrawCommand::ScreenCircle { center: Vec2DScreen { x: 0.0, y: -80.0 }, radius: 10.0, color: None, fill: Some(Color { r: 148, g: 148, b: 148, a: 127 }), thickness: 1.0 },
            DrawCommand::Path { color: Some(Color::gray()), fill: None, commands: vec![
                MoveTo(Vec2DWorld { x: 80.0, y: 20.0 }), LineTo(Vec2DWorld { x: 80.0, y: 20.0 })
            ], thickness: 1.0 },
            DrawCommand::Path { color: Some(Color::gray()), fill: None, commands: vec![
                MoveTo(Vec2DWorld { x: 80.0, y: 0.0 }), LineTo(Vec2DWorld { x: 80.0, y: 20.0 })
            ], thickness: 1.0 },
            DrawCommand::ScreenCircle { center: Vec2DScreen { x: 80.0, y: 0.0 }, radius: 10.0, color: None, fill: Some(Color { r: 148, g: 148, b: 148, a: 127 }), thickness: 1.0 },
        ]);
    }
}
