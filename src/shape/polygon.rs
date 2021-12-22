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
        point: Vec2DWorld,
        handle: Vec2DWorld,
    },

    /// Transition to this after a release event
    FreePoint(Vec2DWorld),

    /// End is detected on the pressed event but this event doesn't return a
    /// finished shape, so this state is transitioned to on pressed and then
    /// detected on relesease to return the built shape
    End(Vec2DWorld),
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
        let init_vec = Vec::with_capacity(8);

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

                self.tip = BuilderState::Handle { point: p, handle: pos };
            }
            BuilderState::PointStart(p) => {
                self.tip = BuilderState::Handle { point: p, handle: pos };
            }
            BuilderState::FreePoint(_) => {
                self.tip = BuilderState::FreePoint(pos);
            }
            BuilderState::Handle { point, .. } => {
                self.tip = BuilderState::Handle { point, handle: pos };
            }
            BuilderState::End(_) => {}
        }
    }

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) {
        match self.tip {
            BuilderState::FreePoint(p) => {
                if let Some(point) = self.touches_prev_point(pos, t, snap) {
                    self.points.push(PathCommand::LineTo(point));
                    self.tip = BuilderState::End(point);
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
            BuilderState::Init(p) | BuilderState::PointStart(p) => {
                self.points.push(PathCommand::LineTo(p));
                self.tip = BuilderState::FreePoint(pos);

                ShapeFinished::No
            }
            BuilderState::Handle { point, handle } => {
                self.points.push(PathCommand::CurveTo(CubicBezierCurve {
                    pt1: handle,
                    pt2: point,
                    to: point,
                }));

                self.tip = BuilderState::FreePoint(pos);

                ShapeFinished::No
            }
            BuilderState::FreePoint(_) => ShapeFinished::No,
            BuilderState::End(_) => {
                ShapeFinished::Yes(Box::new(Path::from_parts(
                    self.points.clone(),
                    self.color,
                    self.thickness,
                )))
            }
        }
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        let mut path_commands = self.points.clone();

        match self.tip {
            BuilderState::Init(_) => {}
            BuilderState::Handle { point, handle } => {
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
            BuilderState::End(p) => {
                path_commands.push(PathCommand::LineTo(p));
            }
        }

        let mut commands = Vec::with_capacity(self.points.len() * 2);

        commands.push(DrawCommand::Path {
            commands: path_commands,
            thickness: self.thickness,
            color: self.color,
        });

        commands.extend(self.points.iter().map(|p| {
            DrawCommand::ScreenCircle {
                center: t.to_screen_coordinates(p.to()),
                radius: snap,
                color: Color::gray(),
                thickness: 1.0,
            }
        }));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAP: f64 = 10.0;

    #[test]
    fn it_behaves() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 0.0), t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 0.0), t, SNAP) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 20.0), t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 20.0), t, SNAP) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP) {
            panic!()
        }
    }

    #[test]
    fn no_clunky_path_ends() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(1.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_mouse_moved(Vec2DScreen::new(1.0, 1.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(31.0, 0.0), t, SNAP);
        poly.handle_button_released(Vec2DScreen::new(30.0, 0.0), t, SNAP);

        poly.handle_mouse_moved(Vec2DScreen::new(29.0, 0.0), t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(33.0, 0.0), t, SNAP) {
            panic!()
        }

        assert_eq!(poly.draw_commands(t, SNAP)[0], DrawCommand::Path {
            color: Default::default(),
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
            color: Default::default(),
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
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }

        // at zoomed in scale closes at shorter distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(2.0, center);
        poly.handle_mouse_moved(p1, t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }

        // at zoomed out scale closes at greater distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(0.5, center);
        poly.handle_mouse_moved(p1, t, SNAP);
        if let ShapeFinished::No = poly.handle_button_released(p1, t, SNAP) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t, SNAP);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t, SNAP) {
            panic!();
        }
    }
}
