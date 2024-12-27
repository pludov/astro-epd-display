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
) -> Result<Option<SystemTime>, Error> {
    // Render the template
    let (yaml, next) = templater::render(state, SystemTime::now())?;
    let primitives = renderer::parse(yaml)?;

    // Then draw it

    let mut display = FrameBuf::<Color, &mut BinaryFrameBuffer<Color>>::new(
        buffer,
        buffer.width() as usize,
        buffer.height() as usize,
    );
    display.clear(Color::default()).unwrap();

    renderer::draw(&mut display, &primitives)?;

    Ok(next)
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

    let mut previous = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
    let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
    let mut force_full_render = true;
    let mut asleep = false;

    loop {
        let state = state::get_state();
        // FIXME: this render must produce a buffer, the buffer must be compared, then only
        // the redraw must be done
        let rendered = render(state, &mut buffer);
        let sleep_limit;
        if rendered.is_err() {
            println!("Error rendering: {:?}", rendered.err());
            sleep_limit = None;
            // FIXME: do something more clever...
        } else {
            // FIXME : return errors
            if force_full_render || buffer.updated(&mut previous) {
                if asleep {
                    device.wake_up().expect("wakeup failed");
                    asleep = false;
                }
                device.update(buffer.buffer()).unwrap();
                if force_full_render {
                    force_full_render = false;
                    buffer.updated(&mut previous);
                }
            }
            sleep_limit = rendered.ok().flatten();
        }

        let max_sleep = match sleep_limit {
            None => None,
            Some(t) => t
                .duration_since(SystemTime::now())
                .ok()
                .or(Some(Duration::from_secs(0))),
        };

        let steps = if asleep {
            &[max_sleep] as &[Option<Duration>]
        } else {
            &[Some(Duration::from_millis(50)), max_sleep]
        };

        for (step_id, step) in steps.iter().enumerate() {
            // Wait for a signal
            if step_id > 0 && !asleep {
                println!("Sleeping device");
                device.sleep().expect("sleep failed");
                asleep = true;
            }
            match if step.is_none() {
                signal.recv().or(Err(RecvTimeoutError::Disconnected))
            } else {
                signal.recv_timeout(step.unwrap())
            } {
                Err(RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
                Ok(_) => {
                    // Signal received, do nothing
                    println!("Signal received");
                    break;
                }
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
