
use embedded_graphics::{mono_font::MonoTextStyleBuilder, prelude::*, text::{Baseline, Text, TextStyleBuilder}};
use embedded_graphics::Drawable;
use embedded_hal::delay::DelayNs;
use epd_waveshare::{epd2in9::*, epd2in9::Epd2in9, prelude::*};
use linux_embedded_hal::{gpio_cdev::{Chip, LineHandle, LineRequestFlags}, spidev::{SpiModeFlags, SpidevOptions}, CdevPin, Delay, SpidevDevice};
use rpi_embedded::gpio::Gpio;

// EPD_BUSY_PIN    = 24;
// EPD_DC_PIN      = 25;
// EPD_RST_PIN     = 17;

// EPD_PWR_PIN     = 18;
// EPD_CS_PIN      = 8;  => il arrive pas à l'avoir en vrai
// EPD_MOSI_PIN    = 10;
// EPD_SCLK_PIN    = 11;

fn demo()  {


    let mut chip = Chip::new("/dev/gpiochip4").unwrap();

    let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();
    spi.configure(
        &SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(10_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build(),
    ).unwrap();
    let busy = CdevPin::new(chip.get_line(24).unwrap().request(LineRequestFlags::INPUT , 0, "busy").unwrap()).unwrap();
    let rst = CdevPin::new(chip.get_line(17).unwrap().request(LineRequestFlags::OUTPUT, 0, "rst").unwrap()).unwrap();
    // let busy = CdevPin::new((24).unwrap().into_input();
    let dc = CdevPin::new(chip.get_line(25).unwrap().request(LineRequestFlags::OUTPUT, 0, "dc").unwrap()).unwrap();
    let power = CdevPin::new(chip.get_line(18).unwrap().request(LineRequestFlags::OUTPUT, 0, "power").unwrap()).unwrap();


    let mut delay = Delay {};
    power.set_value(1).unwrap();
    println!("creating new epd\n");
    rst.set_value(1).unwrap();
    delay.delay_ms(200);
    // Setup the epd
    let mut epd4in2 =
        Epd2in9::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");

    println!("clearing frame\n");
    epd4in2.clear_frame(&mut spi, &mut delay).expect("clear frame failed");
    println!("cleared frame");
    // Setup the graphics
    let mut display = Display2in9::default();
    display.clear(Color::White).unwrap();
    // Build the style
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::Black)
        // .background_color(Color::Black)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let _ = Text::with_text_style("It's working-WoB!", Point::new(5, 5), style, text_style)
        .draw(&mut display);

    epd4in2.update_frame(&mut spi, display.buffer(), &mut delay).unwrap();
    epd4in2
        .display_frame(&mut spi, &mut delay)
        .expect("display frame new graphics");

    // Going to sleep
    epd4in2.sleep(&mut spi, &mut delay).expect("sleep failed");
}


fn main() {
    demo();
}