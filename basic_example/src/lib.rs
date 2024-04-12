use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};
use wasm_bindgen::prelude::*;
use web_sys::console;

use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::{Point, Primitive, WebColors, *},
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    Drawable,
};
use embedded_text::TextBox;

use tinybmp::Bmp;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let document = web_sys::window()
        .expect("could not get window")
        .document()
        .expect("could not get document");
    let body = document.body().expect("could not get document body");

    // for simplicity reasons, this example uses `cargo-run-wasm`, which doesn't allow
    // custom html - so it's augmented here inline. In a real project, you'd likely use `trunk` instead.
    body.set_inner_html(
        r#"
    <header>
    Embedded Graphics Web Simulator!
  </header>

  <div id="custom-container"></div>
  <footer>
    🦀 A rust-embedded x rust-wasm experiment 🦀
    <br />Made using
    <a href="https://github.com/jamwaffles" target="_blank">@jamwaffles'</a>
    <a href="https://github.com/embedded-graphics/simulator" target="_blank">Embedded Graphics</a>
  </footer>
    "#,
    );

    const WIDTH: u32 = 296;
    const HEIGHT: u32 = 128;

    let output_settings = OutputSettingsBuilder::new()
        .scale(1)
        .pixel_spacing(1)
        .build();
    let mut text_display = WebSimulatorDisplay::new((WIDTH, HEIGHT), &output_settings, None);
    let mut img_display = WebSimulatorDisplay::new(
        (128, 128),
        &output_settings,
        document.get_element_by_id("custom-container").as_ref(),
    );

    {
        let display = &mut text_display;
        display.clear(BinaryColor::On).expect("clear");
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

        // Bounding box for our text. Fill it with the opposite color so we can read the text.
        let bounds = Rectangle::new(Point::new(157, 10), Size::new(WIDTH - 157, 0));
        bounds
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(display)
            .unwrap();

        // Create the text box and apply styling options.
        let text = "Embassy\nMy name is\nJohn Aughey";
        let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

        let border_stroke = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::Off)
            .stroke_width(3)
            .stroke_alignment(StrokeAlignment::Outside)
            .build();

        // Draw the text box.
        text_box.draw(display).unwrap();
        text_box
            .bounding_box()
            .into_styled(border_stroke)
            .draw(display)
            .unwrap();
        const FERRIS_IMG: &[u8; 2622] = include_bytes!("../../badge/ferris_1bpp.bmp");

        // Draw ferris
        let tga: Bmp<BinaryColor> = Bmp::from_slice(FERRIS_IMG).unwrap();
        let image = Image::new(&tga, Point::zero());
        let _ = image.draw(display);
        let _ = display.flush();
    }

    //let style = MonoTextStyle::new(&FONT_6X9, Rgb565::CSS_WHITE);

    // if Text::new("Hello, wasm world!", Point::new(10, 30), style)
    //     .draw(&mut text_display)
    //     .is_err()
    // {
    //     console::log_1(&"Couldn't draw text".into());
    // }
    // text_display.flush().expect("could not flush buffer");

    // Load the BMP image
    let bmp = Bmp::from_slice(include_bytes!("../assets/rust-pride.bmp")).unwrap();
    let image = Image::new(&bmp, Point::new(32, 32));
    if image.draw(&mut img_display).is_err() {
        console::log_1(&"Couldn't draw image".into());
    }

    if Circle::new(Point::new(29, 29), 70)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_WHITE, 1))
        .draw(&mut img_display)
        .is_err()
    {
        console::log_1(&"Couldn't draw circle".into());
    }

    img_display.flush().expect("could not flush buffer");

    Ok(())
}
