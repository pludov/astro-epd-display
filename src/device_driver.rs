use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle};
use embedded_graphics_framebuf::FrameBuf;
use serde_json::{json, Value};

use crate::{
    binary_change_tracker::BinaryChangeTracker,
    binary_framebuffer::{BinarisedColor, BinaryFrameBuffer},
    error::Error,
    renderer::{self, container::ShiftedDisplay, ColorFromTemplate},
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

    let mut shifted_display = ShiftedDisplay::from(&mut display);

    renderer::draw(&mut shifted_display, &primitives).map_err(Error::DrawingError)?;

    Ok(next)
}

pub enum RefreshSignal {
    Normal,
    Full,
}

pub trait Device {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn sleep(&mut self) -> Result<(), Error>;
    fn wake_up(&mut self) -> Result<(), Error>;
    fn update(&mut self, buffer: &[u8]) -> Result<(), Error>;
    fn partial_update(&mut self, buffer: &[u8], _rects: &Vec<Rectangle>) -> Result<(), Error> {
        self.update(buffer)
    }
}

// This runs a thread
pub fn drive_device(
    device: &mut dyn Device,
    signal: Receiver<RefreshSignal>,
    max_partial_per_pixel: u8,
) {
    let size = Size {
        width: device.width(),
        height: device.height(),
    };
    println!("Size: {size}\n");

    state::merge_state(
        json!({"width": size.width, "height": size.height}),
        RefreshSignal::Normal,
    )
    .expect("Merging size must succeed");

    let mut previous = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
    let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);

    let mut change_tracker = BinaryChangeTracker::new(size.width, size.height, 8);
    let mut force_full_render = true;
    let mut asleep = false;

    change_tracker.reset(&buffer, &mut previous);

    'driver: loop {
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
            let mut changed_rects = Vec::new();
            if force_full_render
                || change_tracker.update(&buffer, &mut previous, &mut changed_rects)
            {
                if asleep {
                    device.wake_up().expect("wakeup failed");
                    asleep = false;
                }

                if change_tracker.get_max_changes() > max_partial_per_pixel {
                    force_full_render = true;
                }

                if force_full_render {
                    device.update(buffer.buffer()).expect("refresh failed");
                } else {
                    device
                        .partial_update(buffer.buffer(), &changed_rects)
                        .expect("refresh partial failed");
                }
                if force_full_render {
                    force_full_render = false;
                    change_tracker.reset(&buffer, &mut previous);
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
                    println!("Signal disconnected");
                    break 'driver;
                }
                Ok(RefreshSignal::Normal) => {
                    println!("Signal received");
                    break;
                }
                Ok(RefreshSignal::Full) => {
                    force_full_render = true;
                    println!("Full signal received");
                    break;
                }
            }
        }

        // Fetch more pending signals
        while let Ok(s) = signal.try_recv() {
            match s {
                RefreshSignal::Normal => {
                    println!("Signal dequeud");
                }
                RefreshSignal::Full => {
                    force_full_render = true;
                    println!("Full Signal dequeud");
                }
            }
        }
    }

    if !asleep {
        device.sleep().expect("sleep failed");
    }

    println!("Device driver stopped");
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
