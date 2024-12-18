use crate::{
    binary_framebuffer::BinaryFrameBuffer,
    device_driver::{drive_device, Device},
    error::Error,
    renderer::to_display_string,
};
use embedded_graphics::pixelcolor::BinaryColor;
use std::sync::mpsc::Receiver;

pub struct StdoutDevice {
    buffer: BinaryFrameBuffer<BinaryColor>,
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
        println!("{}", to_display_string(&self.buffer, None));
        Ok(())
    }
}

pub fn drive_stdout(signal: Receiver<()>, width: u32, height: u32) {
    let mut device = StdoutDevice {
        buffer: BinaryFrameBuffer::<BinaryColor>::new(width, height),
    };

    drive_device(&mut device, signal)

    // let size = Size{width: epd4in2.width(), height: epd4in2.height()};
    // println!("Size: {size}\n");

    // println!("clearing frame\n");

    // println!("cleared frame");

    // let mut previous = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
    // let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);

    // loop {
    //     let state = state::get_state();
    //     let rendered = render(state, &mut buffer);
    //     if rendered.is_err() {
    //         println!("Error rendering: {:?}", rendered.err());
    //         // FIXME: do something more clever...
    //     } else {
    //         epd4in2.update_frame(&mut spi, buffer.buffer(), &mut delay).unwrap();
    //         epd4in2
    //             .display_frame(&mut spi, &mut delay).unwrap();
    //     }

    //     // Wait for a signal
    //     match signal.recv_timeout(Duration::from_millis(50)) {
    //         Err(RecvTimeoutError::Timeout) => {
    //             epd4in2.sleep(&mut spi, &mut delay).expect("sleep failed");
    //             if signal.recv().is_err() {
    //                 break;
    //             }
    //             epd4in2.wake_up(&mut spi, &mut delay).expect("wakeup failed");
    //         },
    //         Err(RecvTimeoutError::Disconnected) => {
    //             break;
    //         },
    //         Ok(_) => {
    //             // Signal received, do nothing
    //         },
    //     }
    // }
}
