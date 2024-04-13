#![no_std]

use embedded_graphics::geometry::Dimensions as _;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Circle, PrimitiveStyleBuilder, StrokeAlignment};
use embedded_graphics::Drawable;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use tinybmp::Bmp;

pub fn draw_display(
    display: &mut impl DrawTarget<Color = BinaryColor>,
) -> Result<(), &'static str> {
    display.clear(BinaryColor::On).map_err(|_| "clear")?;

    let (width, height) = {
        let bb = display.bounding_box();
        (bb.size.width, bb.size.height)
    };

    // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
    let character_style = MonoTextStyle::new(
        &FONT_10X20,
        // FONT_9X18_BOLD,
        BinaryColor::Off,
    );
    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Left)
        .vertical_alignment(VerticalAlignment::Middle)
        .paragraph_spacing(0)
        .build();

    // Bounding box for our text. Fill it with the opposite color so we can read the text.
    const V_PADDING: i32 = 10;
    let tga: Bmp<BinaryColor> = Bmp::from_slice(FERRIS_IMG).unwrap();
    let image = Image::new(&tga, Point::zero());

    let left_edge = image.bounding_box().size.width;
    let bounds = Rectangle::new(
        Point::new(left_edge.try_into().map_err(|_| "left edge")?, V_PADDING),
        Size::new(
            width.saturating_sub(left_edge),
            height.saturating_sub(V_PADDING.try_into().map_err(|_| "vpadding")?),
        ),
    );

    // bounds
    //     .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
    //     .draw(display)
    //     .map_err(|_| "text box")?;

    // Create the text box and apply styling options.
    let text = "Embassy\nMy name\nJohn Aughey";
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

    // Draw the text box.
    text_box.draw(display).map_err(|_| "draw text box")?;
    // text_box
    //     .bounding_box()
    //     .into_styled(border_stroke)
    //     .draw(display)
    //     .map_err(|_| "draw text box border")?;
    const FERRIS_IMG: &[u8; 2622] = include_bytes!("../../badge/ferris_1bpp.bmp");

    // Draw ferris
    image.draw(display).map_err(|_| "draw ferris")?;
    Ok(())
}
