mod binary_change_tracker;
mod binary_framebuffer;
mod cli;
mod debug;
mod device_driver;
mod epd_driver;
mod error;
mod renderer;
mod scraper;
mod state;
mod stdout_driver;
mod templater;

use axum::{response::Html, routing::get, Router};
use clap::Parser;
use cli::Args;
use device_driver::RefreshSignal;
use scraper::start_scraper;
use serde_json::json;
use std::sync::mpsc::{Receiver, Sender};
use tokio::signal::unix::{signal, SignalKind};
use tokio::{select, signal};

use std::cell::RefCell;
use std::net::SocketAddr;
use std::{panic, process};

use tokio::{net::TcpListener, task};

// EPD_BUSY_PIN    = 24;
// EPD_DC_PIN      = 25;
// EPD_RST_PIN     = 17;

// EPD_PWR_PIN     = 18;
// EPD_CS_PIN      = 8;  => il arrive pas à l'avoir en vrai
// EPD_MOSI_PIN    = 10;
// EPD_SCLK_PIN    = 11;

// fn _demo()  {

//     let mut chip = Chip::new("/dev/gpiochip4").unwrap();

//     let mut spi = SpidevDevice::open("/dev/spidev0.0").unwrap();
//     spi.configure(
//         &SpidevOptions::new()
//             .bits_per_word(8)
//             .max_speed_hz(10_000_000)
//             .mode(SpiModeFlags::SPI_MODE_0)
//             .build(),
//     ).unwrap();
//     let busy = CdevPin::new(chip.get_line(24).unwrap().request(LineRequestFlags::INPUT , 0, "busy").unwrap()).unwrap();
//     let rst = CdevPin::new(chip.get_line(17).unwrap().request(LineRequestFlags::OUTPUT, 0, "rst").unwrap()).unwrap();
//     // let busy = CdevPin::new((24).unwrap().into_input();
//     let dc = CdevPin::new(chip.get_line(25).unwrap().request(LineRequestFlags::OUTPUT, 0, "dc").unwrap()).unwrap();
//     let power = CdevPin::new(chip.get_line(18).unwrap().request(LineRequestFlags::OUTPUT, 0, "power").unwrap()).unwrap();

//     let mut delay = Delay {};
//     power.set_value(1).unwrap();
//     println!("creating new epd\n");
//     rst.set_value(1).unwrap();
//     delay.delay_ms(200);
//     // Setup the epd
//     let mut epd4in2 =
//         Epd2in9::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");

//     println!("clearing frame\n");
//     epd4in2.clear_frame(&mut spi, &mut delay).expect("clear frame failed");
//     println!("cleared frame");
//     // Setup the graphics
//     let mut display = Display2in9::default();
//     display.clear(Color::White).unwrap();
//     // Build the style
//     let style = MonoTextStyleBuilder::new()
//         .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
//         .text_color(Color::Black)
//         // .background_color(Color::Black)
//         .build();
//     let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
//     let _ = Text::with_text_style("It's working-WoB!", Point::new(5, 5), style, text_style)
//         .draw(&mut display);

//     epd4in2.update_frame(&mut spi, display.buffer(), &mut delay).unwrap();
//     epd4in2
//         .display_frame(&mut spi, &mut delay)
//         .expect("display frame new graphics");

//     // Going to sleep
//     epd4in2.sleep(&mut spi, &mut delay).expect("sleep failed");
// }

thread_local! {
    static DRAW_SIGNAL: RefCell<Option<Sender<RefreshSignal>>> = RefCell::new(None);
}

pub fn trigger_draw(kind: RefreshSignal) {
    DRAW_SIGNAL.with(|cell| match cell.borrow().clone() {
        None => {
            return;
        }
        Some(sender) => {
            match sender.send(kind) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error sending draw signal: {:?}", e);
                }
            };
        }
    });
}

async fn run_server(sender: Sender<RefreshSignal>, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    DRAW_SIGNAL.with(|signal| {
        signal.replace(Some(sender.clone()));
    });
    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await.unwrap();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    let app = state::route(app);
    let app = templater::route(app);
    let app = debug::route(app);

    let mut sigint = signal(SignalKind::terminate()).unwrap();
    select! {
        _ = signal::ctrl_c() => {
            println!("Shutting down");
        }
        _ = sigint.recv() => {
            println!("Shutting down from sigterm");
        }
        r = axum::serve(listener, app) => {
            r.unwrap();
            println!("Server stopped by itself ?");
        }
    }
    // FIXME: now adjust the
    state::merge_state(json!({"status": "done"}), RefreshSignal::Full).unwrap();

    DRAW_SIGNAL.with(|signal| {
        signal.replace(None);
    });
}

async fn root() -> Html<&'static str> {
    Html(
        r#"
    <html>
        <head>
            <title>EPD Display</title>
        </head>
        <body>
            <h1>EPD Display</h1>
            <p>Click <a href="/state">here</a> to see the current state</p>
            <p>Click <a href="/template">here</a> to see the current template</p>
            <p>Click <a href="/display">here</a> to see the current display</p>
        </body>
    </html>
    "#,
    )
}

fn run_device(receiver: Receiver<RefreshSignal>, args: &Args) {
    match args.driver.clone() {
        Some(cli::Driver::Epd(epd_config)) => epd_driver::drive_epd(receiver, &epd_config),
        None | Some(cli::Driver::Stdout) => {
            stdout_driver::drive_stdout(receiver, args.width, args.height)
        }
    }
}

async fn load_default_template(args: &Args) {
    if let Some(template) = &args.template {
        let template = tokio::fs::read_to_string(template)
            .await
            .expect(format!("Error loading template: {:?}", template).as_str());

        templater::post_template(template)
            .await
            .expect("Error installing default template");
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        println!("Process paniced");
        process::exit(1);
    }));

    load_default_template(&args).await;

    let (sender, receiver) = std::sync::mpsc::channel::<RefreshSignal>();

    let driver = std::thread::spawn({
        let args = args.clone();
        move || {
            run_device(receiver, &args);
        }
    });

    let local = task::LocalSet::new();
    local
        .run_until(async move {
            start_scraper(&args);
            run_server(sender, args.port).await
        })
        .await;
    driver.join().unwrap();
}
