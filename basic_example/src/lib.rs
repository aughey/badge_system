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

    let display = &mut text_display;
    badge_draw::draw_display(display).expect("could not draw display");
    display.flush().expect("could not flush buffer");

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
