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
use embassy_time::Timer; //r

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
    let mut display = ST7567::new(_dcx, _blx, _rst, _display_spi);

    display.init().unwrap();

    // implementation of embedded graphics hello-world example
    // https://github.com/embedded-graphics/examples/blob/main/eg-0.7/examples/hello-world.rs
    let mut count = 0;
    display.backlight(BacklightStatus::On).unwrap();
    //ignoring st7567::DisplayErrors::outOfBounds
    loop {
        if count <= 128 {
            count += 1;
        } else {
            count = 1;
        }
        for x in 0..128 {
            let _ = display.set_pixel(x, count, true);
            let _ = display.set_pixel(x, 63 - count, true);
        }
        for y in 0..64 {
            let _ = display.set_pixel(count, y, true);
            let _ = display.set_pixel(127 - count, y, true);
        }
        display.show().unwrap();
        Timer::after_millis(50).await;
        display.clear().unwrap();
    }
}
