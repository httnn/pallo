use web_time::Instant;

use crate::{
    platform::{Clipboard, InputType, PlatformCommon},
    *,
};

#[derive(Clone)]
pub struct TextBuilder {
    font_size: f32,
    typeface: usize,
    text: String,
    color: Color,
    variables: Vec<FontVariable>,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self { font_size: 14.0, typeface: 0, text: "".into(), color: Default::default(), variables: vec![] }
    }
}

impl TextBuilder {
    pub fn build<A: App>(mut self, cx: &mut Cx<A>) -> Text {
        if !self.variables.iter().any(|v| v.get_axis() == "wght") {
            self.variables.push(FontVariable::new("wght", A::default_font_weight()));
        }
        let mut text = Text {
            blob: None,
            font: cx.backend.create_font(self.typeface, self.font_size, self.variables),
            text: self.text,
            color: self.color,
        };
        text.set_text(text.text.clone());
        text
    }

    pub fn font_size(mut self, value: f32) -> Self {
        self.font_size = value;
        self
    }

    pub fn typeface(mut self, value: impl Into<usize>) -> Self {
        self.typeface = value.into();
        self
    }

    pub fn color(mut self, value: Color) -> Self {
        self.color = value;
        self
    }

    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn variation(mut self, axis: &'static str, value: f32) -> Self {
        self.variables.push(FontVariable::new(axis, value));
        self
    }
}

pub struct Text {
    font: Font,
    blob: Option<TextBlob>,
    text: String,
    color: Color,
}

impl Text {
    pub fn set_text(&mut self, text: String) -> &mut Self {
        self.text = text;
        self.blob = TextBlob::new(self.text.clone(), &self.font);
        self
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    fn get_glyph_widths(&self, text: &String) -> Vec<f32> {
        self.font.get_glyph_widths(text)
    }

    pub fn get_width(&self) -> f32 {
        self.font.get_string_width(&self.text)
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_cap_height(&self) -> f32 {
        self.font.get_cap_height()
    }

    pub fn draw(&self, canvas: &mut Canvas, bounds: Rect) {
        if let Some(blob) = &self.blob {
            canvas.fill(self.color).draw_text(blob, bounds.relative_point((0.0, 1.0)));
        }
    }

    pub fn get_bounds(&self) -> Rect {
        let w = self.get_width();
        let text_bounds: Rect = Rect::from_xywh(0.0, 0.0, w, self.get_cap_height());
        text_bounds.with_width(w)
    }

    pub fn split_into_lines(&self, max_width: f32) -> Vec<Text> {
        let mut row_width = 0.0;
        let mut current_row = String::new();
        let mut rows = vec![];
        for word in self.text.split(' ') {
            row_width += self.font.get_string_width(word);
            if row_width > max_width {
                rows.push(current_row.clone());
                current_row.clear();
                row_width = 0.0;
            }
            current_row += &(word.to_owned() + " ");
            row_width += self.font.get_string_width(" ");
        }

        rows.push(current_row.clone());

        rows.into_iter()
            .map(|text| Text {
                blob: TextBlob::new(text.clone(), &self.font),
                font: self.font.clone(),
                color: self.color,
                text,
            })
            .collect()
    }
}

pub struct Label {
    pub id: ComponentId,
    text_signal: Computed<String>,
    color: Computed<Color>,
    text: Text,
    text_bounds: Rect,
    x_align: Align,
    y_align: Align,
}

impl Label {
    pub fn new<A: App>(cx: &mut Cx<A>, id: ComponentId, font_size: f32, typeface: impl Into<usize>) -> Self {
        Self::new_with_builder(cx, id, TextBuilder::default().font_size(font_size).typeface(typeface.into()))
    }

    pub fn new_with_builder<A: App>(cx: &mut Cx<A>, id: ComponentId, builder: TextBuilder) -> Self {
        Self {
            id,
            text_signal: builder.text.clone().into(),
            color: builder.color.into(),
            text: builder.build(cx),
            text_bounds: Rect::default(),
            x_align: Align::Center,
            y_align: Align::Center,
        }
    }

    fn get_aligned_text_bounds<A: App>(&self, cx: &mut Cx<A>) -> Rect {
        let bounds = cx.get_bounds(&self.id);
        self.text_bounds.x_aligned_within(bounds, self.x_align).y_aligned_within(bounds, self.y_align)
    }

    pub fn set_text(&mut self, text: impl Into<Computed<String>>) -> &mut Self {
        self.text_signal = text.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<Computed<String>>) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_color(&mut self, color: impl Into<Computed<Color>>) {
        self.color = color.into();
    }

    pub fn with_color(mut self, color: impl Into<Computed<Color>>) -> Self {
        self.set_color(color);
        self
    }

    pub fn with_x_align(mut self, align: Align) -> Self {
        self.x_align = align;
        self
    }

    pub fn get_text_width(&self) -> f32 {
        self.text.get_width()
    }

    pub fn is_empty(&self) -> bool {
        self.text_signal.get().is_empty()
    }

    pub fn set_x_align(&mut self, align: Align) {
        self.x_align = align;
    }

    pub fn set_y_align(&mut self, align: Align) {
        self.y_align = align;
    }

    fn update_text<A: App>(&mut self, cx: &mut Cx<A>) {
        if let Some(text) = self.text_signal.next() {
            self.text.set_text(text);
            let bounds = self.text.get_bounds();
            if bounds != self.text_bounds {
                self.text_bounds = bounds;
                self.notify_size_changed(cx);
            }
        }
    }
}

impl<A: App> Component<A> for Label {
    fn draw(&self, cx: &mut Cx<A>, canvas: &mut Canvas) {
        if !cx.is_visible(&self.id) {
            return;
        }
        let text_bounds = self.get_aligned_text_bounds(cx);
        self.text.draw(canvas, text_bounds);
    }

    fn event(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) {
        if event.update() {
            if let Some(color) = self.color.next() {
                self.text.set_color(color);
            }
            self.update_text(cx);
        }
    }

    fn layout(&mut self, cx: &mut Cx<A>, bounds: Rect) {
        cx.set_bounds(&self.id, bounds);
        self.text_bounds = self.text.get_bounds();
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn get_preferred_size(&mut self, cx: &mut Cx<A>, _parent_bounds: Rect) -> (Option<f32>, Option<f32>) {
        self.update_text(cx);
        (Some(self.text.get_width() + 2.0), None)
    }
}

pub type CharMapper = fn(&String, &String, i32) -> Option<String>;

pub const NUMBER_INPUT_CHAR_MAPPER: CharMapper = |_text, char, _caret| {
    let mut ch = char.to_lowercase().chars().next().unwrap();
    if ch == ',' {
        ch = '.';
    }
    if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '-' {
        return Some(char.clone());
    }
    None
};

pub const NAME_INPUT_CHAR_MAPPER: CharMapper = |_text, char, _caret| {
    let mut ch = char.to_lowercase().chars().next().unwrap();
    if ch == ',' {
        ch = '.';
    }
    if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == ' ' || ch == '/' {
        return Some(char.clone());
    }
    None
};

pub struct TextInput {
    pub label: Label,
    caret_index: i32,
    caret_position: f32,
    anchor_index: i32,
    anchor_position: f32,
    caret_animation_counter: f32,
    start_edit_time: Instant,
    is_editing: Signal<bool>,
    pub edited_text: Signal<String>,
    map_char: CharMapper,
    x_scroll_offset: f32,
    prompt_value: Later<String>,
    is_editable: bool,
    readonly: bool,
    input_type: InputType,
}

impl TextInput {
    pub fn new<A: App>(cx: &mut Cx<A>, id: ComponentId, font_size: f32, typeface: impl Into<usize>) -> Self {
        Self::new_with_builder(cx, id, TextBuilder::default().font_size(font_size).typeface(typeface.into()))
    }

    pub fn new_with_builder<A: App>(cx: &mut Cx<A>, id: ComponentId, builder: TextBuilder) -> Self {
        cx.set_interactive(&id, true);
        Self {
            label: Label::new_with_builder(cx, id, builder),
            caret_index: 0,
            caret_position: 0.0,
            anchor_index: 0,
            anchor_position: 0.0,
            caret_animation_counter: 0.0,
            start_edit_time: Instant::now(),
            is_editing: cx.signal_default(),
            edited_text: cx.signal_default(),
            map_char: NUMBER_INPUT_CHAR_MAPPER,
            x_scroll_offset: 0.0,
            prompt_value: Default::default(),
            is_editable: true,
            readonly: false,
            input_type: InputType::Text,
        }
    }

    pub fn set_text(&mut self, text: impl Into<Computed<String>>) -> &mut Self {
        let editing = self.is_editing.clone();
        let edited_text = self.edited_text.clone();
        let text: Computed<String> = text.into();
        self.edited_text.set(text.get());
        self.label.set_text(editing.cx().computed(move || if editing.get() { edited_text.get() } else { text.get() }));
        self
    }

    pub fn get_text(&self) -> &String {
        self.label.text.get_text()
    }

    pub fn with_text(mut self, text: impl Into<Computed<String>>) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_color(&mut self, color: impl Into<Computed<Color>>) {
        self.label.set_color(color)
    }

    pub fn with_color(mut self, color: impl Into<Computed<Color>>) -> Self {
        self.set_color(color);
        self
    }

    pub fn with_type(mut self, t: InputType) -> Self {
        self.input_type = t;
        self
    }

    pub fn with_x_align(mut self, align: Align) -> Self {
        self.label.set_x_align(align);
        self
    }

    pub fn with_char_mapper(mut self, mapper: CharMapper) -> Self {
        self.map_char = mapper;
        self
    }

    fn get_cursor_x(&self, index: i32) -> f32 {
        self.label.text.get_glyph_widths(&self.edited_text.get_fast())[0..index as usize].iter().sum()
    }

    fn get_cursor_index(&self, position: f32) -> i32 {
        let text = self.edited_text.get_fast();
        let mut min_distance = f32::MAX;
        let mut min_distance_index = 0;
        let mut p = 0.0;
        for (i, width) in self.label.text.get_glyph_widths(&text).iter().enumerate() {
            let dist = (p - position).abs();
            if dist < min_distance {
                min_distance = dist;
                min_distance_index = i;
            }
            p += width;
        }
        if (p - position).abs() < min_distance {
            min_distance_index = text.len();
        }
        min_distance_index as i32
    }

    pub fn get_text_width(&self) -> f32 {
        self.label.get_text_width()
    }

    pub fn is_empty(&self) -> bool {
        self.label.is_empty()
    }

    pub fn is_editing(&self) -> bool {
        self.is_editing.get_fast()
    }

    pub fn set_editable(&mut self, editable: bool) {
        self.is_editable = editable;
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }

    fn update_caret_positions(&mut self) {
        self.caret_position = self.get_cursor_x(self.caret_index);
        self.anchor_position = self.get_cursor_x(self.anchor_index);
    }

    fn move_caret(&mut self, position: i32, move_anchor: bool) {
        self.caret_animation_counter = 0.0;
        self.caret_index = position.clamp(0, self.edited_text.get_fast().len() as i32);
        if move_anchor {
            self.anchor_index = self.caret_index;
        }
        self.update_caret_positions();
    }

    fn remove_selected_text(&mut self) {
        let start = self.anchor_index.min(self.caret_index);
        let end = self.anchor_index.max(self.caret_index);
        self.edited_text.mutate(|mut text| {
            text.replace_range((start as usize)..(end as usize), "");
        });
        self.move_caret(
            if self.anchor_index < self.caret_index {
                self.anchor_index
            } else {
                self.caret_index
            },
            true,
        );
    }

    fn start_edit<A: App>(&mut self, #[allow(unused)] cx: &mut Cx<A>) {
        if self.is_editable {
            self.start_edit_time = Instant::now();
            self.is_editing.set(true);
            self.edited_text.set(self.label.text.get_text().clone());
            #[allow(unused)]
            let val = self.prompt_value.clone();
            #[cfg(target_os = "ios")]
            {
                cx.platform.open_prompt(
                    "Edit value".into(),
                    "Enter".into(),
                    self.edited_text.get(),
                    self.input_type,
                    &self.prompt_value,
                );
            }
            self.select_all();
        }
    }

    pub fn stop_edit(&mut self) {
        self.is_editing.set(false);
    }

    pub fn start_edit_with_text<A: App>(&mut self, cx: &mut Cx<A>, text: impl Into<String>) {
        self.start_edit(cx);
        let text: String = text.into();
        let len = text.len();
        self.edited_text.set(text);
        self.set_cursor_position(len as i32);
        self.focus(cx);
    }

    fn get_aligned_text_bounds<A: App>(&self, cx: &mut Cx<A>) -> Rect {
        self.label.get_aligned_text_bounds(cx)
    }

    pub fn set_x_align(&mut self, align: Align) {
        self.label.set_x_align(align)
    }

    pub fn set_y_align(&mut self, align: Align) {
        self.label.set_y_align(align)
    }

    pub fn select_all(&mut self) {
        self.anchor_index = 0;
        self.caret_index = self.edited_text.get_fast().len() as i32;
        self.update_caret_positions();
    }

    pub fn set_cursor_position(&mut self, pos: i32) {
        self.anchor_index = pos.clamp(0, self.edited_text.get_fast().len() as i32);
        self.caret_index = self.anchor_index;
        self.update_caret_positions();
    }

    pub fn event<A: App>(&mut self, cx: &mut Cx<A>, event: &mut Event<A>) -> Option<String> {
        self.label.event(cx, event);
        match event {
            Event::Update => {
                self.caret_animation_counter += cx.frame_delta_ms * 0.01;

                let safe_margin = 2.0;

                let bounds = self.get_bounds(cx);
                if self.label.text_bounds.width() < bounds.width() {
                    self.x_scroll_offset = 0.0;
                } else {
                    let pos = self.get_aligned_text_bounds(cx).left() + self.caret_position;
                    let min_scroll_offset =
                        0.0_f32.min(-(self.label.text_bounds.width() - bounds.width()) - safe_margin);

                    let offset_left = bounds.left() + safe_margin - pos;
                    let offset_right = bounds.right() - safe_margin - pos;

                    self.x_scroll_offset = offset_left.max(offset_right.min(self.x_scroll_offset));
                    self.x_scroll_offset = min_scroll_offset.max(self.x_scroll_offset);
                }

                if let Some(val) = self.prompt_value.value() {
                    self.is_editing.set(false);
                    return Some(val.to_string());
                }
            }
            Event::FocusChanged(_) => {
                if !self.is_editing.get_fast() && self.is_focused(cx) {
                    self.start_edit(cx);
                } else if self.is_editing.get_fast() && !self.is_focused(cx) {
                    self.is_editing.set(false);
                }
            }
            Event::PointerDown(pointer) => {
                if self.is_hovered(pointer) {
                    let time_since_edit_start = (Instant::now() - self.start_edit_time).as_millis();
                    if time_since_edit_start > 50 && self.is_editing.get_fast() {
                        if cx.num_clicks.is_multiple_of(2) {
                            self.select_all();
                        } else {
                            let text_bounds = self.get_aligned_text_bounds(cx);
                            let x = pointer.position.x - text_bounds.left() - self.x_scroll_offset;
                            let cursor_index = self.get_cursor_index(x);
                            self.move_caret(cursor_index, true);
                        }
                    } else if !self.is_editing.get_fast() {
                        self.start_edit(cx);
                    }
                }
            }
            Event::PointerMove(pointer) => {
                if self.is_pressed(pointer) {
                    let time_since_edit_start = (Instant::now() - self.start_edit_time).as_millis();
                    if time_since_edit_start > 400 && self.is_editing.get_fast() {
                        let text_bounds = self.get_aligned_text_bounds(cx);
                        let x = pointer.position.x - text_bounds.left() - self.x_scroll_offset;
                        let cursor_index = self.get_cursor_index(x);
                        self.move_caret(cursor_index, false);
                    }
                }
            }
            Event::WindowFocusChanged(is_focused) => {
                if !*is_focused && self.is_editing.get_fast() {
                    self.is_editing.set(false);
                    return Some(self.edited_text.get_fast());
                }
            }
            Event::Keydown { key, captured } => {
                if self.is_focused(cx) {
                    match key {
                        Key::Enter => {
                            *captured = true;
                            if !self.is_editing.get_fast() {
                                self.start_edit(cx);
                            } else {
                                self.is_editing.set(false);
                                return Some(self.edited_text.get_fast());
                            }
                        }
                        Key::Escape => {
                            self.is_editing.set(false);
                            *captured = true;
                        }
                        Key::ArrowLeft => {
                            self.move_caret(if cx.mods.meta { 0 } else { self.caret_index - 1 }, !cx.mods.shift);
                            *captured = true;
                        }
                        Key::ArrowRight => {
                            self.move_caret(
                                if cx.mods.meta {
                                    self.get_text().len() as i32
                                } else {
                                    self.caret_index + 1
                                },
                                !cx.mods.shift,
                            );
                            *captured = true;
                        }
                        Key::ArrowUp => {
                            self.move_caret(self.edited_text.get_fast().len() as i32, !cx.mods.shift);
                            *captured = true;
                        }
                        Key::ArrowDown => {
                            self.move_caret(0, !cx.mods.shift);
                            *captured = true;
                        }
                        Key::Backspace => {
                            if self.anchor_index != self.caret_index {
                                self.remove_selected_text();
                            } else if self.caret_index > 0 && !self.readonly {
                                let mut text = self.edited_text.get_fast();
                                text.remove(self.caret_index as usize - 1);
                                self.edited_text.set(text);
                                self.move_caret(self.caret_index - 1, true);
                            }
                            *captured = true;
                        }
                        Key::Character(ch) => {
                            if ch == "v" && cx.mods.meta && !self.readonly {
                                if let Some(txt) = cx.platform.clipboard().read_string() {
                                    self.edited_text.set(txt);
                                    *captured = true;
                                }
                            } else if ch == "c" && cx.mods.meta {
                                let start = self.caret_index.min(self.anchor_index) as usize;
                                let end = self.caret_index.max(self.anchor_index) as usize;
                                let text = (&self.edited_text.get_fast())[start..end].to_owned();
                                cx.platform.clipboard().write_string(text);
                            } else if !self.readonly {
                                let text = self.edited_text.get_fast();
                                if let Some(ch) = (self.map_char)(&text, ch, self.caret_index) {
                                    if !self.is_editing.get_fast() {
                                        self.start_edit(cx);
                                    }
                                    if self.anchor_index != self.caret_index {
                                        self.remove_selected_text();
                                    }
                                    let mut text = self.edited_text.get_fast();
                                    text.insert_str(self.caret_index as usize, &ch);
                                    self.edited_text.set(text);
                                    self.move_caret(self.caret_index + 1, true);
                                    *captured = true;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Key::Enter = key
                    && self.is_focused(cx)
                    && !self.is_editing.get_fast()
                {
                    self.start_edit(cx);
                }
            }
            _ => {}
        }
        None
    }
}

impl<A: App> Component<A> for TextInput {
    fn draw(&self, cx: &mut Cx<A>, canvas: &mut Canvas) {
        if !self.is_visible(cx) {
            return;
        }
        canvas.with_clip_rect(self.label.get_bounds(cx), |canvas| {
            let text_bounds = self.get_aligned_text_bounds(cx);
            self.label.text.draw(canvas, text_bounds.with_x_offset(self.x_scroll_offset));

            if self.is_editing.get_fast() {
                let caret_pos = self.caret_position + self.x_scroll_offset;
                let anchor_pos = self.anchor_position + self.x_scroll_offset;
                let caret_bounds = self.get_aligned_text_bounds(cx).with_expansion(Expansion::y(4.0));
                if self.anchor_index != self.caret_index {
                    canvas.fill(rgba(0xffffff33)).draw_rect(
                        caret_bounds
                            .with_left(caret_bounds.left() + caret_pos.min(anchor_pos))
                            .with_right(caret_bounds.left() + caret_pos.max(anchor_pos)),
                    );
                }
                canvas
                    .stroke(rgb(0xffffff).with_alpha(self.caret_animation_counter.cos() * 0.5 + 0.5), 1.0)
                    .draw_rect(caret_bounds.with_x_offset(caret_pos).with_width(0.0).rounded());
            }
        });
    }

    fn layout(&mut self, cx: &mut Cx<A>, bounds: Rect) {
        self.label.layout(cx, bounds);
    }

    fn get_preferred_size(&mut self, cx: &mut Cx<A>, parent_bounds: Rect) -> (Option<f32>, Option<f32>) {
        self.label.get_preferred_size(cx, parent_bounds)
    }

    fn id(&self) -> &ComponentId {
        &self.label.id
    }
}
