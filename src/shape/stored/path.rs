use xml::writer::{EventWriter, XmlEvent};

use crate::draw_commands::DrawCommand;
use crate::geom::bbox_from_points;
use crate::path_command::PathCommand;
use crate::point::{Unit, Vec2D, WorldUnit};
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;

/// Name, in the pizarra namespace, of the attribute that numbers the `<path>`
/// elements a pressure stroke is saved as, so that loading can join them back
/// into one shape.
pub(crate) const PRESSURE_SEGMENT_ATTR: &str = "pressure-segment";

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand<WorldUnit>>,
    /// Absolute stroke width of each segment (between consecutive commands)
    /// of a pressure-sensitive stroke. `None` means the single width in
    /// `style` applies to the whole path.
    segment_widths: Option<Vec<WorldUnit>>,
    style: Style<WorldUnit>,
}

impl Path {
    pub fn from_parts(commands: Vec<PathCommand<WorldUnit>>, style: Style<WorldUnit>) -> Path {
        Path {
            commands,
            segment_widths: None,
            style,
        }
    }

    /// A pressure stroke with the given width at each command. Each segment
    /// takes the average width of its two ends.
    pub fn from_parts_with_widths(
        commands: Vec<PathCommand<WorldUnit>>,
        style: Style<WorldUnit>,
        widths: Vec<WorldUnit>,
    ) -> Path {
        let segment_widths = (widths.len() == commands.len() && commands.len() >= 2).then(|| {
            widths
                .windows(2)
                .map(|ends| ((ends[0].val() + ends[1].val()) / 2.0).into())
                .collect()
        });
        Path {
            commands,
            segment_widths,
            style,
        }
    }

    /// Joins the segments a pressure stroke was saved as back into one
    /// stroke. Each segment keeps its own width.
    pub(crate) fn from_pressure_segments(segments: Vec<Path>) -> Option<Path> {
        let first = segments.first()?;
        let mut commands = vec![PathCommand::MoveTo(first.commands.first()?.to())];
        let mut segment_widths = Vec::new();

        for segment in &segments {
            let width = segment.style.stroke.map(|s| s.size).unwrap_or_default();
            for command in segment.commands.iter().skip(1) {
                commands.push(*command);
                segment_widths.push(width);
            }
        }

        let mut path = Path {
            commands,
            segment_widths: Some(segment_widths),
            style: first.style,
        };
        // The stroke's own width is the widest segment, which is what hit
        // testing and the bounding box need.
        path.style.stroke = path.style.stroke.map(|stroke| crate::style::Stroke {
            size: path.max_width().unwrap_or(stroke.size),
            ..stroke
        });
        Some(path)
    }

    fn max_width(&self) -> Option<WorldUnit> {
        if let Some(widths) = &self.segment_widths {
            return widths
                .iter()
                .copied()
                .reduce(|a, b| a.val().max(b.val()).into());
        }
        self.style.stroke.map(|s| s.size)
    }

    fn segment_style(&self, width: WorldUnit) -> Style<WorldUnit> {
        Style {
            stroke: self.style.stroke.map(|stroke| crate::style::Stroke {
                size: width,
                ..stroke
            }),
            fill: self.style.fill,
        }
    }

    /// Pressure strokes are drawn as one path per segment, each with its own
    /// width. Round caps and joins hide the seams.
    fn segments_with_widths(&self) -> Vec<(Vec<PathCommand<WorldUnit>>, Style<WorldUnit>)> {
        let Some(widths) = &self.segment_widths else {
            return vec![(self.commands.clone(), self.style)];
        };

        self.commands
            .windows(2)
            .zip(widths)
            .map(|(commands, width)| {
                (
                    vec![PathCommand::MoveTo(commands[0].to()), commands[1]],
                    self.segment_style(*width),
                )
            })
            .collect()
    }
}

impl ShapeStored for Path {
    fn draw_commands(&self) -> Vec<DrawCommand> {
        self.segments_with_widths()
            .into_iter()
            .map(|(commands, style)| DrawCommand::Path { commands, style })
            .collect()
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [Vec2D<WorldUnit>; 2] {
        let points = self.commands.iter().flat_map(|c| c.points());
        let [topleft, bottomright] = bbox_from_points(points);
        let (topleftpad, bottomrightpad) = if let Some(size) = self.max_width() {
            (Vec2D::new(-size, -size), Vec2D::new(size, size))
        } else {
            Default::default()
        };

        [topleft + topleftpad, bottomright + bottomrightpad]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Vec2D<WorldUnit>, radius: WorldUnit) -> bool {
        let mut last_point = None;
        let radius = if let Some(size) = self.max_width() {
            radius + size / 2.0
        } else {
            radius
        };

        for c in self.commands.iter() {
            if c.intersects_circle(center, radius, last_point) {
                return true;
            }

            last_point = Some(c.to());
        }

        false
    }

    fn style(&self) -> Style<WorldUnit> {
        self.style
    }

    fn serialize(&self, writer: &mut EventWriter<&mut Vec<u8>>) {
        // SVG has no variable stroke width, so a pressure stroke is written as
        // one path per segment, like it is rendered. Numbering the segments
        // lets loading join them back into one shape.
        let pressure = self.segment_widths.is_some();
        let attr_name = format!("pizarra:{PRESSURE_SEGMENT_ATTR}");
        for (index, (commands, style)) in self.segments_with_widths().into_iter().enumerate() {
            let d: String = commands.iter().map(|p| p.to_string()).collect();
            let style = format!(
                "fill:{fill};stroke-width:{stroke};stroke-linecap:round;stroke-linejoin:round;stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10;",
                fill = style.fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                stroke = style.stroke.map(|s| s.size).unwrap_or_else(|| 0.0.into()),
                color = style.stroke.map(|s| s.color.css()).unwrap_or_else(|| "none".into()),
                alpha = style.stroke.map(|s| s.color.float_alpha()).unwrap_or(1.0),
            );
            let index = index.to_string();

            let mut element = XmlEvent::start_element("path")
                .attr("style", &style)
                .attr("d", &d);
            if pressure {
                element = element.attr(attr_name.as_str(), &index);
            }
            writer.write(element).unwrap();
            writer.write(XmlEvent::end_element()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::path_command::{CubicBezierCurve, PathCommand::*};
    use crate::style::Stroke;

    use super::*;

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(
            vec![
                MoveTo(Vec2D::new_world(1.0, 0.0)),
                LineTo(Vec2D::new_world(0.0, 1.0)),
            ],
            Style {
                stroke: None,
                fill: None,
            },
        );

        assert_eq!(
            line.bbox(),
            [Vec2D::new_world(0.0, 0.0), Vec2D::new_world(1.0, 1.0)]
        );
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(
            vec![
                MoveTo(Vec2D::new_world(-12.0, -1.0)),
                LineTo(Vec2D::new_world(-5.0, 0.0)),
                LineTo(Vec2D::new_world(-2.0, 7.0)),
                LineTo(Vec2D::new_world(2.0, -8.0)),
            ],
            Style {
                stroke: None,
                fill: None,
            },
        );

        assert_eq!(
            line.bbox(),
            [Vec2D::new_world(-12.0, -8.0), Vec2D::new_world(2.0, 7.0)]
        );
    }

    #[test]
    fn can_delete_single_point_paths() {
        let poly = Path::from_parts(vec![MoveTo(Vec2D::new_world(0.0, 0.0))], Default::default());

        assert!(poly.intersects_circle(Vec2D::new_world(0.0, 0.0), 10.0.into()));
    }

    #[test]
    fn can_delete_beizer_curves() {
        let path = Path::from_parts(
            vec![
                MoveTo(Vec2D::new_world(33.0, 135.0)),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world(50.0, 200.0),
                    pt2: Vec2D::new_world(171.0, 70.0),
                    to: Vec2D::new_world(196.0, 113.0),
                }),
            ],
            Default::default(),
        );

        let cases = [
            (Vec2D::new_world(37.0, 129.0), true),
            (Vec2D::new_world(81.0, 154.0), true),
            (Vec2D::new_world(127.0, 109.0), true),
            (Vec2D::new_world(74.0, 90.0), false),
            (Vec2D::new_world(147.0, 165.0), false),
            (Vec2D::new_world(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0.into()), result);
        }
    }

    #[test]
    fn can_delete_straight_segments() {
        let path = Path::from_parts(
            vec![
                MoveTo(Vec2D::new_world(33.0, 135.0)),
                LineTo(Vec2D::new_world(196.0, 113.0)),
            ],
            Default::default(),
        );

        let cases = [
            (Vec2D::new_world(37.0, 129.0), true),
            (Vec2D::new_world(81.0, 154.0), false),
            (Vec2D::new_world(127.0, 109.0), true),
            (Vec2D::new_world(74.0, 90.0), false),
            (Vec2D::new_world(147.0, 165.0), false),
            (Vec2D::new_world(177.0, 121.0), true),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0.into()), result);
        }
    }

    #[test]
    fn line_thickness_affects_bbox() {
        let path = Path::from_parts(
            vec![
                MoveTo(Vec2D::new_world(0.0, 0.0)),
                LineTo(Vec2D::new_world(30.0, 30.0)),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world(30.0, 60.0),
                    pt2: Vec2D::new_world(60.0, 60.0),
                    to: Vec2D::new_world(60.0, 30.0),
                }),
            ],
            Default::default(),
        );

        let stroke: WorldUnit = Style::default().stroke.unwrap().size;

        assert_eq!(
            path.bbox(),
            [
                Vec2D::new_world(0.0, 0.0) - Vec2D::new(stroke, stroke),
                Vec2D::new_world(60.0, 60.0) + Vec2D::new(stroke, stroke),
            ]
        );
    }

    #[test]
    fn erase_on_touch_stroke() {
        let start = Vec2D::new_world(27.410791, 119.70396);
        let path = Path::from_parts(
            vec![
                MoveTo(start),
                CurveTo(CubicBezierCurve {
                    pt1: start + Vec2D::new_world(0.06519, -10.40955),
                    pt2: start + Vec2D::new_world(11.676337, -10.01372),
                    to: start + Vec2D::new_world(11.966013, 0.48172),
                }),
            ],
            Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: 2.1166.into(),
                }),
                fill: None,
            },
        );

        // these must touch
        assert!(path.intersects_circle(Vec2D::new_world(29.841375, 116.64932), 1.3229166.into()));
        assert!(path.intersects_circle(Vec2D::new_world(33.242252, 110.06812), 1.3229166.into()));
        assert!(path.intersects_circle(Vec2D::new_world(37.421051, 120.87376), 1.3229166.into()));

        // these must touchn't
        assert!(!path.intersects_circle(Vec2D::new_world(39.535831, 112.48697), 1.3229166.into()));
        assert!(!path.intersects_circle(Vec2D::new_world(34.404907, 115.59615), 1.3229166.into()));
        assert!(!path.intersects_circle(Vec2D::new_world(27.536837, 110.98792), 1.3229166.into()));
    }

    #[test]
    #[ignore]
    fn erase_on_touch_interior_if_fill() {
        todo!()
    }
}
