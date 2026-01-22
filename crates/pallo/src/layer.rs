use crate::{
    App, CanvasType, Cx, IntPoint, Point, RasterSurfaceType, Rect, Signal, Surface, renderers,
};

pub struct Layer {
    surface: Option<Surface>,
    target_size: IntPoint,
    surface_size_signal: Signal<IntPoint>,
}

impl Layer {
    pub fn new<A: App>(cx: &mut Cx<A>) -> Self {
        Self {
            surface: None,
            target_size: IntPoint::default(),
            surface_size_signal: cx.signal_default(),
        }
    }

    pub fn update<A: App>(&mut self, cx: &mut Cx<A>) {
        let surface_size =
            (self.target_size.to_float() * cx.scale_factor.get() * cx.ui_scale).to_int();
        if self.surface.is_none() {
            self.surface = Some(Surface::new(surface_size, 1.0));
            self.surface_size_signal.set(surface_size);
        } else if let Some(surface) = &mut self.surface
            && surface.get_size() != surface_size
        {
            *surface = Surface::new(surface_size, 1.0);
            self.surface_size_signal.set(surface_size);
        }
    }

    pub fn draw<A: App>(&self, cx: &mut Cx<A>, canvas: &mut crate::Canvas, position: Point) {
        if let Some(surface) = &self.surface {
            canvas.with_scale(1.0, |canvas| {
                canvas
                    .color(crate::rgb(0xffffff))
                    .draw_surface(surface, position * cx.ui_scale);
            });
        }
    }

    pub fn draw_contents(
        &self,
        draw: impl FnOnce(<crate::renderer::Renderer as renderers::RendererType>::Canvas<'_>, Rect),
    ) {
        if let Some(surface) = &self.surface {
            surface.draw(draw);
        }
    }

    pub fn get_size_computed(&self) -> crate::Computed<IntPoint> {
        self.surface_size_signal.as_computed()
    }

    pub fn resize(&mut self, size: IntPoint) {
        self.target_size = size;
    }
}
