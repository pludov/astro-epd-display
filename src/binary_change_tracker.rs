use embedded_graphics::{
    prelude::{Point, Size},
    primitives::Rectangle,
};
use std::cmp::*;

use crate::binary_framebuffer::BinaryFrameBuffer;

pub struct BinaryChangeTracker {
    width: u32,
    height: u32,
    size: usize,
    buffer: Vec<u8>,
    // The changes will be detected within square of that size
    grain: u32,
    max_changes: u8,
}

impl BinaryChangeTracker {
    pub fn new(width: u32, height: u32, grain: u32) -> Self {
        let size = (width * height) as usize;
        let buffer = vec![0; size];
        BinaryChangeTracker {
            width,
            height,
            grain,
            size,
            buffer,
            max_changes: 0,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_max_changes(&self) -> u8 {
        self.max_changes
    }

    pub fn reset<C>(
        &mut self,
        buffer: &BinaryFrameBuffer<C>,
        reference: &mut BinaryFrameBuffer<C>,
    ) {
        self.max_changes = 0;
        for i in 0..self.size {
            let v = buffer.get_bit(i);
            reference.set_bit(i, v);
            self.buffer[i] = 0;
        }
    }

    fn grain_count(&self, l: u32) -> u32 {
        (l + self.grain - 1) / self.grain
    }

    /// Compare a new frame to the reference one.
    ///  in case of changes, update the reference and return the list of changed rectangles
    ///  the maximum number of changes in a single pixel is stored in `max_changes`
    pub fn update<C>(
        &mut self,
        buffer: &BinaryFrameBuffer<C>,
        reference: &mut BinaryFrameBuffer<C>,
        changed_rects: &mut Vec<Rectangle>,
    ) -> bool {
        if self.width() != buffer.width() || self.height() != buffer.height() {
            panic!("Framebuffers must have the same size");
        }
        if self.width() != reference.width() || self.height() != reference.height() {
            panic!("Framebuffers must have the same size");
        }

        let mut global_changed = false;

        for grain_y in 0..self.grain_count(self.height) {
            let y0 = grain_y * self.grain;
            let y1 = min(y0 + self.grain, self.height);
            for grain_x in 0..self.grain_count(self.width) {
                let x0 = grain_x * self.grain;
                let x1 = min(x0 + self.grain, self.width);

                let mut changed: bool = false;
                let mut min_x = self.width;
                let mut min_y = self.height;
                let mut max_x = 0;
                let mut max_y = 0;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let i = (y * self.width + x) as usize;
                        let v = buffer.get_bit(i);
                        let ref_v = reference.get_bit(i);
                        if v != ref_v {
                            reference.set_bit(i, v);
                            let changes = self.buffer[i] + 1;
                            self.buffer[i] = changes;
                            if self.max_changes < changes {
                                self.max_changes = changes;
                            }

                            if !changed {
                                changed = true;
                                min_x = x;
                                min_y = y;
                                max_x = x;
                                max_y = y;
                            } else {
                                min_x = min(min_x, x);
                                min_y = min(min_y, y);
                                max_x = max(max_x, x);
                                max_y = max(max_y, y);
                            }
                        }
                    }
                }

                if changed {
                    changed_rects.push(Rectangle::new(
                        Point {
                            x: min_x as i32,
                            y: min_y as i32,
                        },
                        Size {
                            width: (max_x - min_x + 1) as u32,
                            height: (max_y - min_y + 1) as u32,
                        },
                    ));
                    global_changed = true;
                }
            }
        }
        global_changed
    }
}
