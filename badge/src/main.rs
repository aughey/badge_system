//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]
#![allow(unreachable_code)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::Timer;
use uc8151::UpdateRegion;
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::StrokeAlignment;
pub mod net;
//use hal::halt;
// The macro for our start-up function

// GPIO traits

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
// use pimoroni_badger2040::entry;
// use pimoroni_badger2040::hal::pac;
// use pimoroni_badger2040::hal::Clock;
// use pimoroni_badger2040::hal;
// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
// use pimoroni_badger2040::hal;
// use pimoroni_badger2040::hal::pac;
// use pimoroni_badger2040::hal::Clock;

use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use static_cell::StaticCell;

use tinybmp::Bmp;

static FERRIS_IMG: &[u8; 2622] = include_bytes!("../ferris_1bpp.bmp");

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, &'static str, 3> = Channel::new();
static LED_RATE_CHANNEL: Signal<CriticalSectionRawMutex, u64> = Signal::new();

enum LedState {
    On,
    Off,
}

use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 32768;
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());
    // Initialize the allocator BEFORE you use it
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }

    // // Grab our singleton objects
    // let mut pac = pac::Peripherals::take().unwrap();
    // // // The single-cycle I/O block controls our GPIO pins
    // let sio = hal::Sio::new(pac.SIO);

    // let empty_status = |_: &str| {};
    // crate::net::main_net(p, spawner, empty_status).await;
    // return;

    // // Set the pins up according to their function on this particular board
    // let pins = pimoroni_badger2040::Pins::new(
    //     embassy_rp::pac::IO_BANK0,
    //     //        pac.IO_BANK0,
    //     embassy_rp::pac::PADS_BANK0,
    //     //pac.PADS_BANK0,
    //     embassy_rp::pac::sio::gpio_bank0,
    //     //        sio.gpio_bank0
    //     &mut pac.RESETS,
    // );
    // Set the LED to be an output
    //let mut led_pin = pins.led.into_push_pull_output();

    // Configure the timer peripheral for our blinky delay
    //let mut timer = HalTimer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut display = {
        // Set up the watchdog driver - needed by the clock setup code
        // let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        // // The default is to generate a 125 MHz system clock
        // let clocks = hal::clocks::init_clocks_and_plls(
        //     pimoroni_badger2040::XOSC_CRYSTAL_FREQ,
        //     pac.XOSC,
        //     pac.CLOCKS,
        //     pac.PLL_SYS,
        //     pac.PLL_USB,
        //     &mut pac.RESETS,
        //     &mut watchdog,
        // )
        // .ok()
        // .unwrap();

        // We use the embassy time Delay so the hal doesn't take control of the timer
        let mut timer = embassy_time::Delay;

        // Set up the pins for the e-ink display
        // let spi_sclk = pins.sclk.into_function::<hal::gpio::FunctionSpi>();
        //        let spi_mosi = pins.mosi.into_function::<hal::gpio::FunctionSpi>();

        let mut spi_cfg = embassy_rp::spi::Config::default();
        spi_cfg.frequency = 12_000_000;
        let clk = p.PIN_18;
        let mosi = p.PIN_19;
        let miso = p.PIN_16;
        let spi = embassy_rp::spi::Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH2, p.DMA_CH3, spi_cfg);

        // let mut dc = pins.inky_dc.into_push_pull_output();
        // let mut cs = pins.inky_cs_gpio.into_push_pull_output();
        // let busy = pins.inky_busy.into_pull_up_input();
        // let reset = pins.inky_res.into_push_pull_output();
        let mut dc = Output::new(p.PIN_20, Level::Low);
        let mut cs = Output::new(p.PIN_17, Level::High);
        let busy = Input::new(p.PIN_26, Pull::Up);
        let reset = Output::new(p.PIN_21, Level::Low);

        // Enable 3.3V power or you won't see anything
        //        let mut power = pins.p3v3_en.into_push_pull_output();
        let mut power = Output::new(p.PIN_10, Level::High);
        power.set_high();

        // let spi = spi.init(
        //     &mut pac.RESETS,
        //     clocks.peripheral_clock.freq(),
        //     RateExtU32::MHz(1),
        //     embedded_hal::spi::MODE_0,
        // );

        dc.set_high(); //.unwrap();
        cs.set_high(); //.unwrap();

        let mut display = uc8151::Uc8151::new(spi, cs, dc, busy, reset);

        // Reset display
        display.reset(&mut timer);

        // Initialise display. Using the default LUT speed setting
        let _ = display.setup(&mut timer, uc8151::LUT::Internal);

        // {
        //     let border_stroke = PrimitiveStyleBuilder::new()
        //         .stroke_color(BinaryColor::Off)
        //         .stroke_width(3)
        //         .stroke_alignment(StrokeAlignment::Outside)
        //         .build();

        //     // Draw a 3px wide outline around the display.
        //     // _ = display
        //     //     .bounding_box()
        //     //     .into_styled(border_stroke)
        //     //     .draw(&mut display);

        //     // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
        //     let character_style = MonoTextStyle::new(
        //         &FONT_10X20,
        //         // FONT_9X18_BOLD,
        //         BinaryColor::Off,
        //     );
        //     let textbox_style = TextBoxStyleBuilder::new()
        //         .height_mode(HeightMode::FitToText)
        //         .alignment(HorizontalAlignment::Center)
        //         .paragraph_spacing(6)
        //         .build();

        //     // Bounding box for our text. Fill it with the opposite color so we can read the text.
        //     let bounds = Rectangle::new(Point::new(157, 10), Size::new(uc8151::WIDTH - 157, 0));
        //     bounds
        //         .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        //         .draw(&mut display)
        //         .unwrap();

        //     // Create the text box and apply styling options.
        //     let text = "Embassy\nMy name is\nJohn Aughey";
        //     let text_box =
        //         TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

        //     // Draw the text box.
        //     text_box.draw(&mut display).unwrap();
        //     text_box
        //         .bounding_box()
        //         .into_styled(border_stroke)
        //         .draw(&mut display)
        //         .unwrap();

        //     // Draw ferris
        //     let tga: Bmp<BinaryColor> = Bmp::from_slice(FERRIS_IMG).unwrap();
        //     let image = Image::new(&tga, Point::zero());
        //     let _ = image.draw(&mut display);
        //     let _ = display.update();
        // }

        badge_draw::draw_display(&mut display, "Initialized").expect("drawed");
        let _ = display.update();
        display
    };

    let mut badge_text = move |text: &str, draw_status: bool| {
        if draw_status {
            display.clear(BinaryColor::On).unwrap();
            let bounds = display.bounding_box();
            let character_style = MonoTextStyle::new(
                &FONT_10X20,
                // FONT_9X18_BOLD,
                BinaryColor::Off,
            );
            let textbox_style = TextBoxStyleBuilder::new()
                .height_mode(HeightMode::FitToText)
                .alignment(HorizontalAlignment::Center)
                .paragraph_spacing(6)
                .build();
            let text_box =
                TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

            text_box.draw(&mut display).unwrap();
            display.update().unwrap();
        } else {
            badge_draw::draw_display(&mut display, text).expect("drawed");
            display.update().unwrap();
        }
    };

    badge_text("Starting net...", true);

    let led = Output::new(p.PIN_22, Level::Low);
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(core1_task(led, &LED_RATE_CHANNEL)));
                unwrap!(spawner.spawn(text_sender()));
            });
        },
    );

    match crate::net::main_net(
        crate::net::NetPins {
            PIN_23: p.PIN_23,
            PIN_25: p.PIN_25,
            PIO0: p.PIO0,
            PIN_24: p.PIN_24,
            PIN_29: p.PIN_29,
            DMA_CH0: p.DMA_CH0,
        },
        spawner,
        &mut badge_text,
        &LED_RATE_CHANNEL,
    )
    .await
    {
        Ok(_) => badge_text("Net done", true),
        Err(e) => badge_text(e, true),
    }

    //    status("DONE");

    return;

    let up_button = Input::new(p.PIN_15, Pull::Up);

    //return;

    info!("Hello from core 0");
    loop {
        // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
        let character_style = MonoTextStyle::new(
            &FONT_10X20,
            // FONT_9X18_BOLD,
            BinaryColor::Off,
        );
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Center)
            .paragraph_spacing(6)
            .build();
        let bounds = Rectangle::new(
            Point::new(157, 0),
            Size::new(uc8151::WIDTH - 157, uc8151::HEIGHT),
        );
        _ = bounds
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display);

        // Create the text box and apply styling options.
        let text = CHANNEL.receive().await;
        let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

        // Draw the text box.
        _ = text_box.draw(&mut display);
        let _region: UpdateRegion = bounds.try_into().unwrap();
        display.partial_update(bounds.try_into().unwrap()).unwrap();
        //display.update().unwrap();
    }
}

#[embassy_executor::task]
async fn core1_task(
    mut led: Output<'static, embassy_rp::peripherals::PIN_22>,
    channel: &'static Signal<CriticalSectionRawMutex, u64>,
) {
    info!("Hello from core 1");
    let mut flash_rate = 500u64;
    loop {
        led.set_high();
        Timer::after_millis(flash_rate).await;

        if let Some(rate) = channel.try_take() {
            flash_rate = rate.clamp(50, 2000);
        }

        led.set_low();
        Timer::after_millis(flash_rate).await;

        if let Some(rate) = channel.try_take() {
            flash_rate = rate.clamp(50, 2000);
        }
    }
}

#[embassy_executor::task]
async fn text_sender() {
    loop {
        _ = CHANNEL.try_send("Hello from\ncore 0");
        Timer::after_millis(5000).await;
        _ = CHANNEL.try_send("Hello again\ncore 0");
        Timer::after_millis(5000).await;
    }
}
