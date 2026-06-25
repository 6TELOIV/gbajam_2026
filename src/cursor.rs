use agb::{
    display::{GraphicsFrame, object::Object}, fixnum::{Num, Rect, Vector2D},
};

use crate::sprites;

// inspired by https://github.com/emmabritton/gba_nonogram_advance/blob/main/src/button_highlight.rs
pub struct CursorHighlight {
    rect: Rect<Num<i32, 4>>,
    target_rect: Rect<Num<i32, 4>>,
    object: Object,
}

impl CursorHighlight {
    /// Create the cursor bordering the given rectangle
    pub fn new(position: Vector2D<i32>, size: Vector2D<i32>) -> Self {
        let rect = Rect::new(position.into(), size.into());
        Self {
            rect,
            target_rect: rect,
            object: Object::new(sprites::CURSOR.sprite(0)),
        }
    }

    /// Start animating the cursor to a new location
    pub fn set_target(&mut self, position: Vector2D<i32>, size: Vector2D<i32>) {
        self.target_rect = Rect::new(position.into(), size.into());
    }

    /// Moves the current cursor location torwards the target location following an s-curve
    pub fn update(&mut self) {
        let factor: Num<i32, 4> = Num::from_f32(0.25);
        self.rect.position += (self.target_rect.position - self.rect.position) * factor;
        self.rect.size += (self.target_rect.size - self.rect.size) * factor;
    }
    /// Draws the 4 corners of the cursor
    pub fn show(&mut self, frame: &mut GraphicsFrame, frame_count: usize) {
        // animate the cursor sprite
        self.object.set_sprite(sprites::CURSOR.animation_sprite(frame_count / 10));

        // place the 4 corners
        let pos = self.rect.position.round();
        let size = self.rect.size.round();
        for idx in 0..4 {
            let idx = idx as i32;
            self.object
                .set_pos(pos + ((size.x - 8) * (idx % 2), (size.y - 8) * (idx / 2)).into());
            self.object.set_hflip((idx % 2) == 1);
            self.object.set_vflip((idx / 2) == 1);
            self.object.show(frame);
        }
    }
}
