use embedded_graphics::prelude::PixelColor;
use embedded_graphics_framebuf::backends::FrameBufferBackend;

pub trait BinarisedColor {
    fn to_binary_color(&self) -> bool;
    fn from_binary_color(value: bool) -> Self;
}

impl BinarisedColor for embedded_graphics::pixelcolor::BinaryColor {
    fn to_binary_color(&self) -> bool {
        match self {
            embedded_graphics::pixelcolor::BinaryColor::On => true,
            embedded_graphics::pixelcolor::BinaryColor::Off => false,
        }
    }

    fn from_binary_color(value: bool) -> Self {
        if value {
            embedded_graphics::pixelcolor::BinaryColor::On
        } else {
            embedded_graphics::pixelcolor::BinaryColor::Off
        }
    }
}

pub struct BinaryFrameBuffer<C> {
    width: u32,
    height: u32,
    size: usize,
    pub buffer: Vec<u8>,
    _color: std::marker::PhantomData<C>,
}

fn bits_to_bytes_size(width: usize) -> usize {
    (width + 7) / 8
}

fn get_bit(index: usize) -> (usize, u8) {
    let byte = index / 8;
    let bit = index % 8;
    let mask = 1 << (7 - bit);
    (byte, mask)
}

impl<C> BinaryFrameBuffer<C> {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let buffer = vec![0; bits_to_bytes_size(size)];
        BinaryFrameBuffer {
            width,
            height,
            size,
            buffer,
            _color: std::marker::PhantomData,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn from_buffer(&mut self, buffer: &[u8]) -> () {
        for i in 0..buffer.len() {
            self.buffer[i] = buffer[i];
        }
    }

    pub fn get_bit(&self, index: usize) -> bool {
        let (byte, mask) = get_bit(index);
        self.buffer[byte] & mask != 0
    }
    pub fn set_bit(&mut self, index: usize, value: bool) {
        let (byte, mask) = get_bit(index);
        if value {
            self.buffer[byte] |= mask;
        } else {
            self.buffer[byte] &= !mask;
        }
    }
}

impl<C: PixelColor + BinarisedColor> FrameBufferBackend for &mut BinaryFrameBuffer<C> {
    type Color = C;

    fn set(&mut self, index: usize, color: Self::Color) {
        let (byte, mask) = get_bit(index);
        if color.to_binary_color() {
            self.buffer[byte] |= mask;
        } else {
            self.buffer[byte] &= !mask;
        }
    }

    /// Returns a pixels color
    fn get(&self, index: usize) -> Self::Color {
        let b = self.get_bit(index);
        C::from_binary_color(b)
    }

    /// Nr of elements in the backend
    fn nr_elements(&self) -> usize {
        self.size
    }
}
