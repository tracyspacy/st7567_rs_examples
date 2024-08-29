#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;

use embedded_hal::spi::{Mode, Phase};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;
use rp2040_hal as hal;
use rp2040_hal::fugit::RateExtU32;
use rp_pico as bsp;
use st7567_rs::{BacklightStatus, ST7567};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    //display setup

    let _cs = pins
        .gpio17
        .into_push_pull_output_in_state(hal::gpio::PinState::High);

    let _sck = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let _mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
    let dcx = pins.gpio20.into_push_pull_output();
    let blx = pins.gpio9.into_push_pull_output();
    let rst = pins.gpio21.into_push_pull_output();

    let mode = Mode {
        phase: Phase::CaptureOnSecondTransition,
        polarity: embedded_hal::spi::Polarity::IdleHigh,
    };

    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (_mosi, _sck));
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        10_000_000u32.Hz(),
        mode,
    );
    let spi_device = ExclusiveDevice::new_no_delay(spi, _cs);
    let mut display = ST7567::new(dcx, blx, rst, spi_device);
    display.init().unwrap();
    let mut count = 0;
    display.backlight(BacklightStatus::On).unwrap();
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
        delay.delay_ms(50);
        display.clear().unwrap();
    }
}

// End of file
