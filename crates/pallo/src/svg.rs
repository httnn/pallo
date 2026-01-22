use std::{rc::Rc, slice::Iter};

use crate::{
    Align, App, Canvas, Cx, Point, Rect, point,
    renderers::{CanvasType, PathType, renderer::Path},
};

#[derive(Debug)]
enum Token {
    Command(char),
    Number(f32),
}

impl Token {
    fn is_relative(&self) -> bool {
        match self {
            Token::Command(c) => c.is_lowercase(),
            Token::Number(_) => false,
        }
    }
}

fn tokenize_svg_path(mut d: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut number = String::new();
    d.push(' '); // make sure last token is added
    for c in d.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '-' | ',' | ' ' => {
                if !number.is_empty()
                    && let Ok(n) = std::mem::take(&mut number).parse::<f32>()
                {
                    tokens.push(Token::Number(n));
                }
            }
            '.' => {
                if number.contains('.')
                    && let Ok(n) = std::mem::take(&mut number).parse::<f32>()
                {
                    tokens.push(Token::Number(n));
                }
            }
            _ => {}
        }
        match c {
            'A'..='Z' | 'a'..='z' => {
                tokens.push(Token::Command(c));
            }
            '0'..='9' | '.' | '-' => {
                number.push(c);
            }
            _ => {}
        }
    }

    tokens
}

#[derive(Debug)]
enum SvgPathCommand {
    MoveTo(Point),
    LineTo(Point),
    EllipticalArc { r: Point, angle: f32, large_arc: bool, sweep_arc: bool, point: Point },
    Bezier { cp1: Point, cp2: Point, point: Point },
    ClosePath,
}

fn get_n_numbers<const N: usize>(iter: &mut Iter<'_, Token>) -> Option<[f32; N]> {
    let mut out = [0.0; N];
    let backup = iter.clone();
    for j in 0..N {
        if let Some(Token::Number(n)) = iter.next() {
            out[j] = *n;
        } else {
            *iter = backup;
            return None;
        }
    }
    Some(out)
}

fn parse_svg_path(d: impl Into<String>) -> Result<Vec<SvgPathCommand>, String> {
    let mut commands = Vec::new();
    let tokens = tokenize_svg_path(d.into());
    let mut iter = tokens.iter();
    let mut p = point(0.0, 0.0);
    while let Some(token) = iter.next() {
        match token {
            Token::Command('M') | Token::Command('m') => {
                let mut first = true;
                while let Some([x, y]) = get_n_numbers(&mut iter) {
                    if token.is_relative() {
                        p += point(x, y);
                    } else {
                        p = point(x, y);
                    }
                    commands.push(if first {
                        SvgPathCommand::MoveTo(p)
                    } else {
                        SvgPathCommand::LineTo(p)
                    });
                    first = false;
                }
            }
            Token::Command('L') | Token::Command('l') => {
                while let Some([x, y]) = get_n_numbers(&mut iter) {
                    if token.is_relative() {
                        p += point(x, y);
                    } else {
                        p = point(x, y);
                    }
                    commands.push(SvgPathCommand::LineTo(p));
                }
            }
            Token::Command('H') | Token::Command('h') => {
                while let Some([value]) = get_n_numbers(&mut iter) {
                    if token.is_relative() {
                        p.x += value;
                    } else {
                        p.x = value;
                    }
                    commands.push(SvgPathCommand::LineTo(p));
                }
            }
            Token::Command('V') | Token::Command('v') => {
                while let Some([value]) = get_n_numbers(&mut iter) {
                    if token.is_relative() {
                        p.y += value;
                    } else {
                        p.y = value;
                    }
                    commands.push(SvgPathCommand::LineTo(p));
                }
            }
            Token::Command('C') | Token::Command('c') => {
                while let Some([cp1_x, cp1_y, cp2_x, cp2_y, pt_x, pt_y]) = get_n_numbers(&mut iter) {
                    let mut cp1 = point(cp1_x, cp1_y);
                    let mut cp2 = point(cp2_x, cp2_y);
                    let pt = point(pt_x, pt_y);
                    if token.is_relative() {
                        cp1 += p;
                        cp2 += p;
                        p += pt;
                    } else {
                        p = pt;
                    }
                    commands.push(SvgPathCommand::Bezier { cp1, cp2, point: p });
                }
            }
            Token::Command('S') | Token::Command('s') => {
                while let Some([cp_x, cp_y, pt_x, pt_y]) = get_n_numbers(&mut iter) {
                    let mut cp = point(cp_x, cp_y);
                    let pt = point(pt_x, pt_y);
                    if token.is_relative() {
                        cp += p;
                        p += pt;
                    } else {
                        p = pt;
                    }
                    commands.push(SvgPathCommand::Bezier {
                        cp1: if let Some(SvgPathCommand::Bezier { cp2, .. }) = commands.last() {
                            *cp2
                        } else {
                            p
                        },
                        cp2: cp,
                        point: p,
                    });
                }
            }
            Token::Command('A') | Token::Command('a') => {
                while let Some([rx, ry, angle, large_arc, sweep_arc, x, y]) = get_n_numbers(&mut iter) {
                    if token.is_relative() {
                        p += point(x, y);
                    } else {
                        p = point(x, y);
                    }
                    commands.push(SvgPathCommand::EllipticalArc {
                        r: point(rx, ry),
                        angle,
                        large_arc: large_arc > 0.5,
                        sweep_arc: sweep_arc > 0.5,
                        point: p,
                    });
                }
            }
            Token::Command('Z') => {
                commands.push(SvgPathCommand::ClosePath);
            }
            Token::Command(c) => {
                panic!("Unknown command {c}");
            }
            _ => {
                panic!("Unhandled number!");
            }
        }
    }

    Ok(commands)
}

struct SvgShape {
    pub(crate) paths: Vec<Path>,
    pub(crate) viewbox: Rect,
}

fn get_shape(svg: &'static str) -> Result<SvgShape, String> {
    let viewbox = {
        let s = svg.find("viewBox=\"").ok_or("No viewbox found.")? + 9;
        let e = s + svg[s..].find('"').ok_or("Incorrect viewbox tag.")?;
        let mut tokens = svg[s..e].split(' ');
        let x = tokens.next().and_then(|v| v.parse::<f32>().ok()).ok_or("Invalid viewbox")?;
        let y = tokens.next().and_then(|v| v.parse::<f32>().ok()).ok_or("Invalid viewbox")?;
        let width = tokens.next().and_then(|v| v.parse::<f32>().ok()).ok_or("Invalid viewbox")?;
        let height = tokens.next().and_then(|v| v.parse::<f32>().ok()).ok_or("Invalid viewbox")?;
        Rect::from_xywh(x, y, width, height)
    };

    let mut paths = vec![];

    let even_odd = svg.contains("fill-rule=\"evenodd\"");

    let mut position = 0;
    while let Some(mut d_start) = svg[position..].find("d=\"").map(|v| v + 3) {
        d_start += position;
        let d_end = d_start + svg[d_start..].find('"').ok_or("Invalid path argument.")?;
        let d = svg[d_start..d_end].to_owned();

        let mut path = Path::default();
        if even_odd {
            path.fill_type_even_odd();
        }
        for cmd in parse_svg_path(d)? {
            match cmd {
                SvgPathCommand::MoveTo(point) => {
                    path.move_to(point);
                }
                SvgPathCommand::LineTo(point) => {
                    path.line_to(point);
                }
                SvgPathCommand::EllipticalArc { r, angle, large_arc, sweep_arc, point } => {
                    path.arc_to_rotated(r, angle, large_arc, sweep_arc, point);
                }
                SvgPathCommand::ClosePath => {
                    path.close();
                }
                SvgPathCommand::Bezier { cp1, cp2, point } => {
                    path.cubic_to(cp1, cp2, point);
                }
            }
        }
        paths.push(path);
        position = d_end;
    }
    Ok(SvgShape { viewbox, paths })
}

pub struct Svg {
    shape: Rc<SvgShape>,
    scale: f32,
    translation: Point,
}

impl Clone for Svg {
    fn clone(&self) -> Self {
        Self { shape: self.shape.clone(), scale: self.scale, translation: self.translation }
    }
}

impl Svg {
    pub fn new(svg: &'static str) -> Self {
        Self { scale: 1.0, translation: Default::default(), shape: get_shape(svg).unwrap().into() }
    }

    pub fn set_bounds<A: App>(&mut self, _cx: &mut Cx<A>, target_rect: Rect) {
        let true_target_rect =
            target_rect.with_aspect_ratio_keep_centered(self.shape.viewbox.width() / self.shape.viewbox.height());
        self.scale = true_target_rect.width() / self.shape.viewbox.width();
        self.translation = -self.shape.viewbox.edge_point(Align::Start, Align::Start)
            + true_target_rect.edge_point(Align::Start, Align::Start);
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        for path in &self.shape.paths {
            canvas
                .save()
                .translate(self.translation)
                .scale_rel(point(self.scale, self.scale))
                .draw_path(path)
                .restore();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::svg::parse_svg_path;

    #[test]
    fn test_tokenizer() {
        let path = "M17 5H7a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2ZM7 2a5 5 0 0 0-5 5v10a5 5 0 0 0 5 5h10a5 5 0 0 0 5-5V7a5 5 0 0 0-5-5H7Z";
        let _ = parse_svg_path(path);
    }
}
