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

struct LayerManager {
    layers: BTreeMap<LayerId, Layer>,
    layer_stack: Vec<LayerId>,
    graphics: Graphics,
}

impl LayerManager {
    fn new(graphics: &Graphics) -> Self {
        Self {
            layers: BTreeMap::new(),
            layer_stack: vec![],
            graphics: *graphics,
        }
    }

    fn register(&mut self, layer: Layer) {
        let id = layer.id();
        self.layers.insert(id, layer);
    }

    fn set_position(&mut self, id: LayerId, pos: usize) {
        if !self.layers.contains_key(&id) {
            // not registered
            return;
        }
        self.layer_stack.retain(|elem| *elem != id);
        let new_pos = usize::min(pos, self.layer_stack.len());
        self.layer_stack.insert(new_pos, id);
    }

    fn draw(&mut self) {
        for id in &self.layer_stack {
            if let Some(layer) = self.layers.get(&id) {
                layer.draw(&mut self.graphics);
            }
        }
    }

    fn move_relative(&mut self, id: LayerId, diff: &Vector2D<isize>) {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.move_relative(diff);
        }
    }
}