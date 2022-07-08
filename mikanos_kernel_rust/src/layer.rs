use crate::window::Window;
use crate::{Graphics, Vector2D};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::option::Option::{None, Some};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerId(usize);

#[derive(Debug)]
pub struct Layer {
    id: LayerId,
    pos: Vector2D<isize>,
    window: Option<Arc<Window>>,
}

impl Layer {
    pub fn new(id: LayerId) -> Self {
        Layer {
            id,
            pos: Vector2D::<isize> { x: 0, y: 0 },
            window: None,
        }
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    pub fn set_window(&mut self, window: Arc<Window>) {
        self.window = Some(window);
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    pub fn move_to(&mut self, pos: &Vector2D<isize>) {
        self.pos = *pos
    }

    pub fn move_relative(&mut self, pos: &Vector2D<isize>) {
        self.pos += *pos
    }

    pub fn draw(&self, graphics: &mut Graphics) {
        if let Some(window) = self.window() {
            window.draw_to(graphics, &self.pos);
        }
    }
}

#[derive(Debug)]
struct LayerManager {
    layers: BTreeMap<LayerId, Layer>,
    layer_stack: Vec<LayerId>,
}

impl LayerManager {
    fn new() -> Self {
        Self {
            layers: BTreeMap::new(),
            layer_stack: vec![],
        }
    }

    fn register(&mut self, layer: Layer) {
        let id = layer.id();
        self.layers.insert(id, layer);
    }

    fn draw(&self, graphic: &mut Graphics) {
        for id in &self.layer_stack {
            if let Some(layer) = self.layers.get(&id) {
                layer.draw(graphic);
            }
        }
    }

    fn move_relative(&mut self, id: LayerId, diff: &Vector2D<isize>) {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.move_relative(diff);
        }
    }
}
