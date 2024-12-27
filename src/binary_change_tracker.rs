use crate::binary_framebuffer::BinaryFrameBuffer;

pub struct BinaryChangeTracker {
    width: u32,
    height: u32,
    size: usize,
    buffer: Vec<u8>,
    max_changes: u8,
}

impl BinaryChangeTracker {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let buffer = vec![0; size];
        BinaryChangeTracker {
            width,
            height,
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

    pub fn update<C>(
        &mut self,
        buffer: &BinaryFrameBuffer<C>,
        reference: &mut BinaryFrameBuffer<C>,
    ) -> bool {
        let mut changed = false;

        if self.width() != buffer.width() || self.height() != buffer.height() {
            panic!("Framebuffers must have the same size");
        }
        if self.width() != reference.width() || self.height() != reference.height() {
            panic!("Framebuffers must have the same size");
        }

        for i in 0..self.size {
            let v = buffer.get_bit(i);
            let ref_v = reference.get_bit(i);
            if v != ref_v {
                reference.set_bit(i, v);
                let changes = self.buffer[i];
                self.buffer[i] = changes + 1;
                if self.max_changes < changes + 1 {
                    self.max_changes = changes + 1;
                }
                changed = true;
            }
        }
        changed
    }
}
