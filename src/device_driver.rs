use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_framebuf::FrameBuf;
use serde_json::Value;

use crate::{
    binary_framebuffer::{BinarisedColor, BinaryFrameBuffer},
    error::Error,
    renderer::{self, ColorFromTemplate},
    state, templater,
};
use std::{
    sync::{
        mpsc::{Receiver, RecvTimeoutError},
        Arc,
    },
    time::{Duration, SystemTime},
};

fn render<Color: PixelColor + BinarisedColor + ColorFromTemplate + Default>(
    state: Arc<Value>,
    buffer: &mut BinaryFrameBuffer<Color>,
) -> Result<(), Error> {
    // Render the template
    let (yaml, _) = templater::render(state, SystemTime::now())?;
    let primitives = renderer::parse(yaml)?;

    // Then draw it

    let mut display = FrameBuf::<Color, &mut BinaryFrameBuffer<Color>>::new(
        buffer,
        buffer.width() as usize,
        buffer.height() as usize,
    );
    display.clear(Color::default()).unwrap();

    renderer::draw(&mut display, &primitives)?;

    Ok(())
}

pub trait Device {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn sleep(&mut self) -> Result<(), Error>;
    fn wake_up(&mut self) -> Result<(), Error>;
    fn update(&mut self, buffer: &[u8]) -> Result<(), Error>;
}

// This runs a thread
pub fn drive_device(device: &mut dyn Device, signal: Receiver<()>) {
    let size = Size {
        width: device.width(),
        height: device.height(),
    };
    println!("Size: {size}\n");

    let mut _previous = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
    let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);

    loop {
        let state = state::get_state();
        let rendered = render(state, &mut buffer);
        if rendered.is_err() {
            println!("Error rendering: {:?}", rendered.err());
            // FIXME: do something more clever...
        } else {
            // FIXME : return errors
            device.update(buffer.buffer()).unwrap();
        }

        // Wait for a signal
        match signal.recv_timeout(Duration::from_millis(50)) {
            Err(RecvTimeoutError::Timeout) => {
                println!("Sleeping");

                device.sleep().expect("sleep failed");
                if signal.recv().is_err() {
                    break;
                }
                println!("Signal received during sleep");
                device.wake_up().expect("wakeup failed");
            }
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
            Ok(_) => {
                // Signal received, do nothing
                println!("Signal received");
            }
        }
    }
    //     // Setup the graphics
    // let mut display = Display2in9::default();

    // // Build the style
    // let style = MonoTextStyleBuilder::new()
    //     .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
    //     .text_color(Color::Black)
    //     // .background_color(Color::Black)
    //     .build();
    // let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    // let _ = Text::with_text_style("It's working-WoB!", Point::new(5, 5), style, text_style)
    //     .draw(&mut display);

    // epd4in2.update_frame(&mut spi, display.buffer(), &mut delay).unwrap();
    // epd4in2
    //     .display_frame(&mut spi, &mut delay)
    //     .expect("display frame new graphics");

    // // Going to sleep
    // epd4in2.sleep(&mut spi, &mut delay).expect("sleep failed");
}
