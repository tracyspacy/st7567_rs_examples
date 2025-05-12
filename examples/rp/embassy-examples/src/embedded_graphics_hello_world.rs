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
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
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

    // implementation of embedded graphics hello-world example
    // https://github.com/embedded-graphics/examples/blob/main/eg-0.7/examples/hello-world.rs

    loop {
        let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
        let border_stroke = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(3)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();
        let fill = PrimitiveStyle::with_fill(BinaryColor::On);
        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        let yoffset = 14;

        // Draw a 3px wide outline around the display.
        display
            .bounding_box()
            .into_styled(border_stroke)
            .draw(&mut display)
            .unwrap();

        Triangle::new(
            Point::new(16, 16 + yoffset),
            Point::new(16 + 16, 16 + yoffset),
            Point::new(16 + 8, yoffset),
        )
        .into_styled(thin_stroke)
        .draw(&mut display)
        .unwrap();

        Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
            .into_styled(fill)
            .draw(&mut display)
            .unwrap();

        Circle::new(Point::new(88, yoffset), 17)
            .into_styled(thick_stroke)
            .draw(&mut display)
            .unwrap();

        let text = "embedded-graphics";

        // Draw centered text.
        Text::with_alignment(
            text,
            display.bounding_box().center() + Point::new(0, 15),
            character_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        display.show().unwrap();

        display.backlight(BacklightStatus::On).unwrap();

        Timer::after_millis(100).await;
        display.clear().unwrap();
    }
}
