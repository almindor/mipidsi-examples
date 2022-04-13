#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::delay::Delay;
use hifive1::hal::e310x::QSPI1;
use hifive1::hal::gpio::{gpio0::*, IOF0, NoInvert, Regular, Output};
use hifive1::hal::spi::{SpiBus, MODE_0, SpiConfig, SpiExclusiveDevice};
use hifive1::pin;
use hifive1::{
    hal::{prelude::*, DeviceResources},
    sprintln,
};
use mipidsi::models::ST7789;
use riscv_rt::entry;

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::primitives::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use mipidsi::{Display, Orientation, DisplayOptions};


fn init() -> (
    Display<SPIInterfaceNoCS<SpiExclusiveDevice<QSPI1, (Pin3<IOF0<NoInvert>>, (), Pin5<IOF0<NoInvert>>)>, Pin13<Output<Regular<NoInvert>>>>, Pin11<Output<Regular<NoInvert>>>, ST7789>,
    Delay,
) {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());
    let mut delay = Delay::new();

    let _backlight = pins.pin10.into_output();
    let rst = pins.pin11.into_output(); // reset pin
    let _cs = pins.pin12.into_output(); // keep low while drivign display
    let dc = pins.pin13.into_output(); // data/clock switch

    let sck = pin!(pins, spi0_sck).into_iof0(); // SPI clock to LCD
    let mosi = pin!(pins, spi0_mosi).into_iof0(); // SPI MOSI to LCD

    // Configure SPI
    let spi_pins = (mosi, (), sck);
    let spi_bus = SpiBus::new(p.QSPI1, spi_pins);

    let spi_config_display = SpiConfig::new(MODE_0, 40.mhz().into(), &clocks);
    let spi_display = spi_bus.new_device(&spi_config_display);

    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(spi_display, dc);

    // create driver
    let mut display = Display::st7789(di, rst);

    // display options
    let mut options = DisplayOptions::default();
    options.orientation = Orientation::Landscape(false);

    // initialize
    display.init(&mut delay, options).unwrap();

    (display, delay)
}

#[entry]
fn main() -> ! {
    // create driver
    let (mut display, mut _delay) = init();

    let circle1 =
        Circle::new(Point::new(128, 64), 64).into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    let circle2 = Circle::new(Point::new(64, 64), 64)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1));

    let blue_with_red_outline = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::BLUE)
        .stroke_color(Rgb565::RED)
        .stroke_width(1) // > 1 is not currently supported in embedded-graphics on triangles
        .build();
    let triangle = Triangle::new(
        Point::new(40, 120),
        Point::new(40, 220),
        Point::new(140, 120),
    )
    .into_styled(blue_with_red_outline);

    let line = Line::new(Point::new(180, 160), Point::new(239, 239))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));

    // draw two circles on black background
    display.clear(Rgb565::BLACK).unwrap();
    circle1.draw(&mut display).unwrap();
    circle2.draw(&mut display).unwrap();
    triangle.draw(&mut display).unwrap();
    line.draw(&mut display).unwrap();

    sprintln!("Rendering done");

    loop {
        continue; // keep optimizer from removing in --release
    }
}
