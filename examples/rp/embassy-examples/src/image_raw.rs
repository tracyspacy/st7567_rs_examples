#![no_std]
#![no_main]

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::Blocking,
};
use embassy_sync::blocking_mutex::NoopMutex;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    pixelcolor::BinaryColor,
    prelude::*,
    Drawable,
};

use st7567_rs::{BacklightStatus, ST7567};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _}; //r

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let _cs = p.PIN_17;
    let _dc = p.PIN_20;
    let _sck = p.PIN_18;
    let _mosi = p.PIN_19;
    let _bl = p.PIN_9;
    let _reset_pin = p.PIN_21;
    let _inner = p.SPI0;

    let mut _rst = Output::new(_reset_pin, Level::Low);
    let mut _blx = Output::new(_bl, Level::Low);
    let mut _dcx = Output::new(_dc, Level::Low);
    let mut _csx = Output::new(_cs, Level::High);
    let mut display_config = spi::Config::default();
    display_config.frequency = 10000000;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh; //?

    //creating spi driver supporting write only
    static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<SPI0, Blocking>>>> = StaticCell::new();
    let spi = Spi::new_blocking_txonly(_inner, _sck, _mosi, display_config.clone());
    let spi_bus = NoopMutex::new(RefCell::new(spi));
    let spi_bus = SPI_BUS.init(spi_bus);
    let mut _display_spi = SpiDeviceWithConfig::new(spi_bus, _csx, display_config);

    // display
    let mut display = ST7567::new(
        _dcx,
        _blx,
        _rst,
        _display_spi,
        st7567_rs::ScreenDirection::Normal,
        st7567_rs::Bias::Bias1_7,
    );

    display.init().unwrap();

    //image drawing https://docs.rs/embedded-graphics/latest/embedded_graphics/image/struct.ImageRaw.html

    let img: ImageRawLE<BinaryColor> = ImageRawLE::new(include_bytes!("../assets/rust.raw"), 64);
    Image::new(&img, Point::new(32, 0))
        .draw(&mut display)
        .unwrap();

    display.show().unwrap();
    display.backlight(BacklightStatus::On).unwrap();
    loop {}
}

// End of file
