use crate::{PointType, Style, ToSvgStr, ViewBox};
use geo_types::{
    CoordNum, Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::NumCast;

impl<T: CoordNum> ToSvgStr for Coordinate<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        Point::from(*self).to_svg_str(style)
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        Point::from(*self).viewbox(style)
    }
}

impl<T: CoordNum> ToSvgStr for Point<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        if let Some(point_type) = style.point_type.clone() {
            match point_type {
            PointType::Text => format!(
                r#"<text class="{}" x="{x:?}" y="{y:?}" {style}>{text}</text>"#,
                class = style.text_classes.clone().unwrap_or("".into()),
                x = self.x(),
                y = self.y(),
                text = style.text.clone().unwrap_or("".into()),
                style = style,
            ),
            PointType::Poi => {
                let (min_x, min_y, vb_width, vb_height) = style.icon_svg_viewbox.unwrap_or((0,0,100,100));
                let (width, height) = style.icon_svg_width_height.unwrap_or((60,60));
                let (x, y) = (format!("{:?}", self.x()).parse::<f64>().unwrap_or(0.0),
                format!("{:?}", self.y()).parse::<f64>().unwrap_or(0.0));

                #[allow(unused_assignments, unused_mut)]
                let mut dbg_cir = "".to_string();

                // dbg_cir = format!(r#"<circle cx="{x:?}" cy="{y:?}" r=10></circle>"#,
                //     x = x,
                //     y = y
                // );

                let text = style.text.clone().and_then(|text|
                    Some(
                        format!(r#"<text x="{x:?}" y="{y:?}">{text}</text>{debug_circle}"#,
                            debug_circle = dbg_cir,
                            x = (x + width as f64 / 2.0 + 15.0),
                            y = (y + height as f64 - 45.0),
                            text = text,
                        )
                    )
                ).unwrap_or("".into());

                format!(
                    r#"<svg x="{x:?}" y="{y:?}" width="{w}" height="{h}" viewBox="{mx} {my} {vbw} {vbh}" {style}>{path}</svg>{text}"#,
                    style = style,
                    path = style.icon_svg_path.clone().unwrap_or("".into()),
                    w = width,
                    h = height,
                    mx = min_x,
                    my = min_y,
                    vbw = vb_width,
                    vbh = vb_height,
                    x = x - (width as f64 / 2.0),
                    y = y - (height as f64 / 2.0),
                    text = text,
                )
            }
            PointType::Symbol |
            PointType::Circle => format!(
                r#"<circle cx="{x:?}" cy="{y:?}" r="{radius}"{style}/>"#,
                x = self.x(),
                y = self.y(),
                radius = style.radius,
                style = style,
            )
            }
        } else {
            format!(
                r#"<circle alt="point_type_none" cx="{x:?}" cy="{y:?}" r="{radius}"{style}/>"#,
                x = self.x(),
                y = self.y(),
                radius = style.radius,
                style = style,
            )
        }
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        let radius = style.radius + style.stroke_width.unwrap_or(1.0);
        ViewBox::new(
            NumCast::from(self.x()).unwrap_or(0f32) - radius,
            NumCast::from(self.y()).unwrap_or(0f32) - radius,
            NumCast::from(self.x()).unwrap_or(0f32) + radius,
            NumCast::from(self.y()).unwrap_or(0f32) + radius,
        )
    }
}

impl<T: CoordNum> ToSvgStr for MultiPoint<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        self.0.iter().map(|point| point.to_svg_str(style)).collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.0.iter().fold(ViewBox::default(), |view_box, point| {
            view_box.add(&point.viewbox(style))
        })
    }
}

impl<T: CoordNum> ToSvgStr for Line<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        format!(
            r#"<path d="M {x1:?} {y1:?} L {x2:?} {y2:?}"{style}/>"#,
            x1 = self.start.x,
            y1 = self.start.y,
            x2 = self.end.x,
            y2 = self.end.y,
            style = style,
        )
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        let style = Style {
            radius: 0.0,
            ..style.clone()
        };
        self.start.viewbox(&style).add(&self.end.viewbox(&style))
    }
}

impl<T: CoordNum> ToSvgStr for LineString<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        let d = self
            .lines()
            .map(|line| {
                format!(
                    "M {x1:?} {y1:?} L {x2:?}  {y2:?}",
                    x1 = line.start.x,
                    y1 = line.start.y,
                    x2 = line.end.x,
                    y2 = line.end.y,
                )
            })
            .reduce(|a, b| format!("{} {}", a, b))
            .unwrap_or("".into());

        let text_part = if let (Some(text), Some(id)) = (style.text.clone(), style.id.clone()) {
            format!(
                r##"<text class="{class}"><textPath xlink:href="#{path_ref}"{start_offset}>{text}<textPath/></text>"##,
                class = style.text_classes.as_ref().unwrap_or(&"".into()),
                path_ref = id,
                text = text,
                start_offset = style
                    .text_start_offset
                    .and_then(|o| Some(format!(r#"startOffset="{}""#, o)))
                    .unwrap_or("".into()),
            )
        } else {
            "".into()
        };

        format!(
            r#"<path d="{d}"{style}/>{txt}"#,
            d = d,
            style = style,
            txt = text_part,
        )
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.lines().fold(ViewBox::default(), |view_box, line| {
            view_box.add(&line.viewbox(style))
        })
    }
}

impl<T: CoordNum> ToSvgStr for MultiLineString<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        self.0
            .iter()
            .map(|line_string| line_string.to_svg_str(style))
            .collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.0
            .iter()
            .fold(ViewBox::default(), |view_box, line_string| {
                view_box.add(&line_string.viewbox(style))
            })
    }
}

impl<T: CoordNum> ToSvgStr for Polygon<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        use std::fmt::Write;
        let mut path = String::new();
        for contour in std::iter::once(self.exterior()).chain(self.interiors().iter()) {
            let mut points = contour.points_iter();
            if let Some(first_point) = points.next() {
                write!(path, "M {:?} {:?}", first_point.x(), first_point.y()).unwrap()
            }
            for point in points {
                write!(path, " L {:?} {:?}", point.x(), point.y()).unwrap();
            }
            write!(path, " Z ").unwrap();
        }

        format!(
            r#"<path fill-rule="evenodd" d="{path}"{style}/>"#,
            path = path,
            style = style,
        )
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.exterior()
            .lines()
            .chain(
                self.interiors()
                    .iter()
                    .flat_map(|interior| interior.lines()),
            )
            .fold(ViewBox::default(), |view_box, line_string| {
                view_box.add(&line_string.viewbox(style))
            })
    }
}

impl<T: CoordNum> ToSvgStr for Rect<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        Polygon::from(*self).to_svg_str(style)
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        Polygon::from(*self).viewbox(style)
    }
}

impl<T: CoordNum> ToSvgStr for Triangle<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        Polygon::new(self.to_array().iter().cloned().collect(), vec![]).to_svg_str(style)
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        Polygon::new(self.to_array().iter().cloned().collect(), vec![]).viewbox(style)
    }
}

impl<T: CoordNum> ToSvgStr for MultiPolygon<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        self.0
            .iter()
            .map(|polygons| polygons.to_svg_str(style))
            .collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.0
            .iter()
            .fold(ViewBox::default(), |view_box, polygons| {
                view_box.add(&polygons.viewbox(style))
            })
    }
}

impl<T: CoordNum> ToSvgStr for Geometry<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        use Geometry::*;
        match self {
            Point(point) => point.to_svg_str(style),
            Line(line) => line.to_svg_str(style),
            LineString(line_tring) => line_tring.to_svg_str(style),
            Triangle(triangle) => triangle.to_polygon().to_svg_str(style),
            Rect(rect) => rect.to_polygon().to_svg_str(style),
            Polygon(polygon) => polygon.to_svg_str(style),
            MultiPoint(multi_point) => multi_point.to_svg_str(style),
            MultiLineString(multi_line_string) => multi_line_string.to_svg_str(style),
            MultiPolygon(multi_polygon) => multi_polygon.to_svg_str(style),
            GeometryCollection(geometry_collection) => geometry_collection.to_svg_str(style),
        }
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        use Geometry::*;
        match self {
            Point(point) => point.viewbox(style),
            Line(line) => line.viewbox(style),
            LineString(line_tring) => line_tring.viewbox(style),
            Triangle(triangle) => triangle.to_polygon().viewbox(style),
            Rect(rect) => rect.to_polygon().viewbox(style),
            Polygon(polygon) => polygon.viewbox(style),
            MultiPoint(multi_point) => multi_point.viewbox(style),
            MultiLineString(multi_line_string) => multi_line_string.viewbox(style),
            MultiPolygon(multi_polygon) => multi_polygon.viewbox(style),
            GeometryCollection(geometry_collection) => geometry_collection.viewbox(style),
        }
    }
}

impl<T: CoordNum> ToSvgStr for GeometryCollection<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        self.0
            .iter()
            .map(|geometry| geometry.to_svg_str(style))
            .collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.0
            .iter()
            .fold(ViewBox::default(), |view_box, geometry| {
                view_box.add(&geometry.viewbox(style))
            })
    }
}

impl<'a, T: ToSvgStr> ToSvgStr for &'a [T] {
    fn to_svg_str(&self, style: &Style) -> String {
        self.iter()
            .map(|geometry| geometry.to_svg_str(style))
            .collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.iter().fold(ViewBox::default(), |view_box, item| {
            view_box.add(&item.viewbox(style))
        })
    }
}

impl<T: ToSvgStr> ToSvgStr for Vec<T> {
    fn to_svg_str(&self, style: &Style) -> String {
        self.iter()
            .map(|geometry| geometry.to_svg_str(style))
            .collect()
    }

    fn viewbox(&self, style: &Style) -> ViewBox {
        self.iter().fold(ViewBox::default(), |view_box, item| {
            view_box.add(&item.viewbox(style))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Color, ToSvg};
    use geo_types::{LineString, Point, Polygon};

    #[test]
    fn test_point() {
        println!(
            "{}",
            Point::new(0.0, 0.0)
                .to_svg()
                .with_fill_color(Color::Named("red"))
                .with_radius(10.0)
                .with_stroke_color(Color::Named("black"))
                .and(
                    Point::new(50.0, 0.0)
                        .to_svg()
                        .with_radius(5.0)
                        .with_stroke_color(Color::Named("blue"))
                )
                .with_stroke_width(1.0)
                .with_opacity(0.5)
                .with_fill_opacity(0.5)
                .with_fill_color(Color::Named("green"))
        );
    }

    #[test]
    fn test_polygon() {
        println!(
            "{}",
            Polygon::new(
                LineString(vec![
                    (210.0, 0.0).into(),
                    (300.0, 0.0).into(),
                    (300.0, 90.0).into(),
                    (210.0, 90.0).into()
                ]),
                vec![LineString(vec![
                    (230.0, 20.0).into(),
                    (280.0, 20.0).into(),
                    (280.0, 70.0).into(),
                    (230.0, 70.0).into()
                ])]
            )
            .to_svg()
            .with_fill_color(Color::Named("black"))
            .with_stroke_color(Color::Named("red"))
        );
    }
}
