use core::fmt;
use std::fmt::{Debug, Formatter};

use embedded_graphics::{
    prelude::{PixelColor, Point, Size},
    Pixel,
};
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

    /// Compute changes in previous and update previous
    /// TODO : return a set of rectangles to update
    pub fn updated(&self, previous: &mut Self) -> bool {
        if self.size != previous.size {
            panic!("Framebuffers must have the same size");
        }

        let mut result = false;
        for i in 0..bits_to_bytes_size(self.size) {
            let new_value = self.buffer[i];
            if new_value != previous.buffer[i] {
                previous.buffer[i] = new_value;
                result = true;
            }
        }
        result
    }

    pub fn dimensions(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
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

impl<C: PixelColor + BinarisedColor> BinaryFrameBuffer<C> {
    pub fn set_pixel(&mut self, x: u32, y: u32, color: C) {
        let index = (y * self.width + x) as usize;
        let (byte, mask) = get_bit(index);
        if color.to_binary_color() {
            self.buffer[byte] |= mask;
        } else {
            self.buffer[byte] &= !mask;
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> C {
        let index = (y * self.width + x) as usize;
        let b = self.get_bit(index);
        C::from_binary_color(b)
    }

    pub fn iter(&self) -> impl IntoIterator<Item = Pixel<C>> + use<'_, C> {
        (0..self.size).map(move |i| {
            let b = self.get_bit(i);
            Pixel(
                Point::new(i as i32 % self.width as i32, i as i32 / self.width as i32),
                C::from_binary_color(b),
            )
        })
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

impl<C> Debug for BinaryFrameBuffer<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Count 0 & 1
        let mut count = [0, 0];
        for i in 0..self.size {
            count[self.get_bit(i) as usize] += 1;
        }
        write!(
            f,
            "BinaryFrameBuffer<BinaryColor>(width: {}, height: {}, 0: {}, 1: {})",
            self.width, self.height, count[0], count[1]
        )
    }
}
