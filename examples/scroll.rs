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
    let (mut display, mut delay) = init();

    // 3 lines composing a big "F"
    let line1 = Line::new(Point::new(100, 20), Point::new(100, 220))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));
    let line2 = Line::new(Point::new(100, 20), Point::new(160, 20))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));
    let line3 = Line::new(Point::new(100, 105), Point::new(160, 105))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));

    // triangle to be shown "in the scroll zone"
    let triangle = Triangle::new(
        Point::new(240, 100),
        Point::new(240, 140),
        Point::new(320, 120),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN));

    // draw the "F" + scroll-section arrow triangle
    display.clear(Rgb565::BLACK).unwrap();
    line1.draw(&mut display).unwrap();
    line2.draw(&mut display).unwrap();
    line3.draw(&mut display).unwrap();
    triangle.draw(&mut display).unwrap();

    sprintln!("Rendering done, scrolling...");

    let mut scroll = 1u16; // absolute scroll offset
    let mut direction = true; // direction
    let scroll_delay = 20u8; // delay between steps
    loop {
        delay.delay_ms(scroll_delay);
        display.set_scroll_offset(scroll).unwrap();

        if scroll % 80 == 0 {
            direction = !direction;
        }

        match direction {
            true => scroll += 1,
            false => scroll -= 1,
        }
    }
}
