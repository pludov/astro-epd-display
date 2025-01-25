use crate::{
    cli::EpdConfig,
    device_driver::{drive_device, Device, RefreshSignal},
    error::Error,
};
use embedded_graphics::primitives::Rectangle;
use embedded_hal::delay::DelayNs;
use epd_waveshare::{epd2in9_v2::Epd2in9, prelude::*};
use linux_embedded_hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};
use std::sync::mpsc::Receiver;

struct EpdDevice {
    epd4in2: Epd2in9<SpidevDevice, CdevPin, CdevPin, CdevPin, Delay>,
    spi: SpidevDevice,
    delay: Delay,
    // Does the device still has in memory the current frame
    memory_content: bool,
    // Last rendered frame. Required for partial update
    current_frame: Option<Box<Vec<u8>>>,
    max_partial: u8,
    cur_partial: u8,
}

impl EpdDevice {
    fn internal_update(&mut self, buffer: &[u8], full: bool) -> Result<(), Error> {
        // Inverse the buffer so default is white
        let new_frame: Box<Vec<u8>> = Box::new(buffer.iter().map(|x| !x).collect());
        // FIXME: rotate

        if !full && self.current_frame.is_some() && self.cur_partial < self.max_partial {
            if !self.memory_content {
                self.epd4in2
                    .update_old_frame(
                        &mut self.spi,
                        &*self.current_frame.as_ref().unwrap(),
                        &mut self.delay,
                    )
                    .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;

                self.memory_content = true;
            }
            self.epd4in2
                .update_new_frame(&mut self.spi, &*new_frame, &mut self.delay)
                .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;

            self.epd4in2
                .display_new_frame(&mut self.spi, &mut self.delay)
                .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;

            self.cur_partial += 1;
        } else {
            // let (w, h) = (self.width(), self.height());
            self.epd4in2
                .update_frame(&mut self.spi, &*new_frame, &mut self.delay)
                .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;
            self.epd4in2
                .display_frame(&mut self.spi, &mut self.delay)
                .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;
            self.memory_content = false;
            self.cur_partial = 0;
        }
        self.current_frame = Some(new_frame);
        Ok(())
    }
}

impl Device for EpdDevice {
    fn width(&self) -> u32 {
        self.epd4in2.width()
    }

    fn height(&self) -> u32 {
        self.epd4in2.height()
    }

    fn sleep(&mut self) -> Result<(), Error> {
        self.epd4in2
            .sleep(&mut self.spi, &mut self.delay)
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;
        Ok(())
    }

    fn wake_up(&mut self) -> Result<(), Error> {
        self.epd4in2
            .wake_up(&mut self.spi, &mut self.delay)
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;
        self.memory_content = false;
        Ok(())
    }

    fn partial_update(&mut self, buffer: &[u8], _rects: &Vec<Rectangle>) -> Result<(), Error> {
        self.internal_update(buffer, false)
    }

    fn update(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.internal_update(buffer, true)
    }
}

pub fn drive_epd(signal: Receiver<RefreshSignal>, config: &EpdConfig) {
    let mut chip = Chip::new("/dev/gpiochip4").unwrap();

    let mut spi = SpidevDevice::open("/dev/spidev1.0").unwrap();
    spi.configure(
        &SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(10_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build(),
    )
    .unwrap();
    let busy = CdevPin::new(
        chip.get_line(24)
            .unwrap()
            .request(LineRequestFlags::INPUT, 0, "busy")
            .unwrap(),
    )
    .unwrap();
    let rst = CdevPin::new(
        chip.get_line(23)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 0, "rst")
            .unwrap(),
    )
    .unwrap();
    // let busy = CdevPin::new((24).unwrap().into_input();
    let dc = CdevPin::new(
        chip.get_line(25)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 0, "dc")
            .unwrap(),
    )
    .unwrap();

    /*let power = CdevPin::new(
        chip.get_line(18)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 0, "power")
            .unwrap(),
    )
    .unwrap();*/

    let mut delay = Delay {};
    // power.set_value(1).unwrap();
    println!("creating new epd\n");
    rst.set_value(1).unwrap();
    delay.delay_ms(200);
    // Setup the epd
    let mut epd4in2: Epd2in9<SpidevDevice, CdevPin, CdevPin, CdevPin, Delay> =
        Epd2in9::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");

    epd4in2
        .clear_frame(&mut spi, &mut delay)
        .expect("clear frame failed");

    let mut epd_device = EpdDevice {
        epd4in2,
        spi,
        delay,
        memory_content: false,
        current_frame: None,
        max_partial: config.max_partial_per_pixel,
        cur_partial: 0,
    };
    drive_device(&mut epd_device, signal, config.max_partial_per_pixel);

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
