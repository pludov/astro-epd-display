use crate::{
    binary_framebuffer::BinaryFrameBuffer,
    device_driver::{drive_device, Device},
    error::Error,
    renderer::to_display_string,
};
use embedded_graphics::{pixelcolor::BinaryColor, primitives::Rectangle};
use std::sync::mpsc::Receiver;

pub struct StdoutDevice {
    buffer: BinaryFrameBuffer<BinaryColor>,
}

impl StdoutDevice {
    fn screen_goto(&self, x: i32, y: i32) {
        let screen_x = x + 1;
        let screen_y = y / 2 + 1;

        print!("\x1B[{};{}f", screen_y, screen_x);
    }
    fn clear_to_bottom(&self) {
        self.screen_goto(0, self.height() as i32);
        print!("\x1B[0J");
    }
    fn update_rect(&self, rect: Rectangle) -> Result<(), Error> {
        // Adjust y0 to an even number
        let y0 = rect.top_left.y / 2 * 2;
        let mut y1 = rect.top_left.y + rect.size.height as i32;
        if y1 % 2 != 0 {
            y1 += 1;
        }

        let rect = Rectangle::new(
            embedded_graphics::prelude::Point {
                x: rect.top_left.x,
                y: y0,
            },
            embedded_graphics::prelude::Size {
                width: rect.size.width,
                height: (y1 - y0) as u32,
            },
        );

        let str = to_display_string(&self.buffer, Some(rect));

        let mut y = rect.top_left.y;
        // self.screen_goto(rect.top_left.x, y);
        //print!("{}", str);

        // Split the string for \n
        for line in str.split('\n') {
            self.screen_goto(rect.top_left.x, y);
            print!("{}", line);
            y += 2;
        }

        Ok(())
    }
}

impl Device for StdoutDevice {
    fn width(&self) -> u32 {
        self.buffer.width()
    }

    fn height(&self) -> u32 {
        self.buffer.height()
    }

    fn sleep(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn wake_up(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn update(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // plain copy
        self.buffer.from_buffer(buffer);

        // Clear screen
        print!("\x1B[2J");

        self.update_rect(Rectangle::new(
            embedded_graphics::prelude::Point { x: 0, y: 0 },
            embedded_graphics::prelude::Size {
                width: self.buffer.width(),
                height: self.buffer.height(),
            },
        ))?;
        self.screen_goto(0, self.height() as i32);

        Ok(())
    }

    fn partial_update(&mut self, buffer: &[u8], rects: &Vec<Rectangle>) -> Result<(), Error> {
        // plain copy
        self.buffer.from_buffer(buffer);

        self.clear_to_bottom();

        for rect in rects {
            self.update_rect(rect.clone())?;
        }
        self.screen_goto(0, self.height() as i32);

        Ok(())
    }
}

pub fn drive_stdout(signal: Receiver<()>, width: u32, height: u32) {
    let mut device = StdoutDevice {
        buffer: BinaryFrameBuffer::<BinaryColor>::new(width, height),
    };

    drive_device(&mut device, signal)
}
