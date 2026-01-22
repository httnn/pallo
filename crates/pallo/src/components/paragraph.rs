use crate::*;

pub struct Paragraph {
    pub id: ComponentId,
    lines: Option<Vec<(Rect, Text)>>,
    height: f32,
    line_height: f32,
    text: TextBuilder,
}

impl Paragraph {
    pub fn new(id: ComponentId, typeface: impl Into<usize>) -> Self {
        Self { id, line_height: 1.5, lines: None, text: TextBuilder::default().typeface(typeface.into()), height: 0.0 }
    }

    pub fn set_font_size(&mut self, value: f32) {
        self.text = self.text.clone().font_size(value);
        self.lines = None;
    }

    pub fn set_text(&mut self, color: Color, text: impl Into<String>) {
        self.text = self.text.clone().text(text).color(color);
        self.lines = None;
    }

    pub fn set_line_height(&mut self, value: f32) {
        self.line_height = value;
    }

    pub fn num_lines<A: App>(&mut self, cx: &mut Cx<A>) -> usize {
        self.update_lines(cx, self.get_bounds(cx));
        self.lines.as_ref().map(|l| l.len()).unwrap_or(0)
    }

    fn update_lines<A: App>(&mut self, cx: &mut Cx<A>, mut bounds: Rect) {
        let top = bounds.top();
        let lines_text = self.text.clone().build(cx).split_into_lines(bounds.width());
        let mut lines = vec![];
        for line in lines_text {
            let cap_height = line.get_cap_height();
            lines.push((bounds.with_height(line.get_cap_height()), line));
            bounds = bounds.with_y_offset(cap_height * self.line_height);
        }
        self.height = bounds.top() - top;
        self.lines = Some(lines);
    }
}

impl<A: App> Component<A> for Paragraph {
    fn draw(&self, _cx: &mut Cx<A>, canvas: &mut Canvas) {
        if let Some(lines) = &self.lines {
            for (bounds, line) in lines {
                line.draw(canvas, *bounds);
            }
        }
    }

    fn event(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) {
        match event {
            Event::Update if self.lines.is_none() => {
                self.relayout(cx);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut Cx<A>, bounds: Rect) {
        cx.set_bounds(&self.id, bounds);
        self.lines = None;
        self.update_lines(cx, bounds);
    }

    fn get_preferred_size(&mut self, cx: &mut Cx<A>, parent_bounds: Rect) -> (Option<f32>, Option<f32>) {
        self.update_lines(cx, parent_bounds);
        (Some(parent_bounds.width()), Some(self.height))
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }
}
