use embedded_graphics::prelude::Primitive;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyle, Rectangle},
};

pub fn draw_display(display: &mut impl DrawTarget<Color = BinaryColor>) {
    display.clear(BinaryColor::Off).expect("clear");

    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    Rectangle::new(Point::new(0, 0), Size::new(128, 128))
        .into_styled(style)
        .draw(display)
        .unwrap();

    Circle::new(Point::new(64, 64), 64)
        .into_styled(style)
        .draw(display)
        .unwrap();
}
