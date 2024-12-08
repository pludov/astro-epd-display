use crate::{
    device_driver::{drive_device, Device},
    error::Error,
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::{epd2in9::Epd2in9, prelude::*};
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
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))
    }

    fn wake_up(&mut self) -> Result<(), Error> {
        self.epd4in2
            .wake_up(&mut self.spi, &mut self.delay)
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))
    }

    fn update(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.epd4in2
            .update_frame(&mut self.spi, buffer, &mut self.delay)
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))?;
        self.epd4in2
            .display_frame(&mut self.spi, &mut self.delay)
            .map_err(|e| Error::HWError(format!("SPI error{:?}", e)))
    }
}

pub fn drive_epd(signal: Receiver<()>) {
    let mut chip = Chip::new("/dev/gpiochip4").unwrap();

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();
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
        chip.get_line(17)
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
    let power = CdevPin::new(
        chip.get_line(18)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 0, "power")
            .unwrap(),
    )
    .unwrap();

    let mut delay = Delay {};
    power.set_value(1).unwrap();
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
    };
    drive_device(&mut epd_device, signal)

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
