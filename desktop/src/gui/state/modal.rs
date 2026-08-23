
use std::sync::LazyLock;

use iced::{Background, Color, Element, Length, Padding, Size};
use iced::border::{Border, Radius};
use iced::font::{Font, Weight};
use iced::widget::{
    center, Container, container,
    Column, column,
    opaque,
    radio,
    space,
    stack,
    Text,
    text
};
use iced::widget::rule::{self, FillMode};

use super::{bold_text, LENGTH_UNIT, Pad, SCREEN_SIZE};

pub fn construct_modal<'a, M>(
    base: impl Into<Element<'a, M>>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M>
where
    M: 'a + Clone,
{
    let Size { width, height } = *MODAL_SIZE;

    let modal = opaque(
        center(
            container(content)
                .center_x(width)
                .center_y(height)
                .padding(*LENGTH_UNIT)
                .style(|_theme| container::background(Color::WHITE)),
        )
        .style(|_theme| {
            container::background(Color {
                a: 0.8,
                ..Color::BLACK
            })
        })
    );

    stack![base.into(), modal].into()
}

pub static MODAL_SIZE: LazyLock<Size> = LazyLock::new(|| {
    let Size { width: screen_width, height: screen_height } = *SCREEN_SIZE;
    let width = screen_width as f32 * 0.52;
    let height = screen_height as f32 * 0.6;

    Size {
        width,
        height
    }
});



pub fn primary_title<'a, M: 'a>(text_str: &'a str) -> Column<'a, M> {
    let text_size = MODAL_SIZE.height / 20.0;

    column![
        title(text_str, text_size),
        horizontal_seperator(100.0)
    ] 
}

pub fn title<'a, M>(text_str: &'a str, text_size: f32) -> Container<'a, M> {
    let title_text = bold_text(text_str).size(text_size);

    title_text.pad(*LENGTH_UNIT)
}

pub fn text_with_padding<'a, M>(text: Text<'a>) -> Container<'a, M> {
    container(text)
        .align_left(Length::Fill)
        .padding(*LENGTH_UNIT)
}

pub fn horizontal_seperator<'a, M: 'a>(fill_percent: f32) -> Container<'a, M> {
    rule::horizontal(1)
        .style(move |theme| rule::Style {
            fill_mode: FillMode::Percent(fill_percent),
            ..rule::default(theme)
        })
        .pad_x(*LENGTH_UNIT)
}

pub fn horizontal_seperator2<'a, M: 'a>() -> Container<'a, M> {
    container(space())
        .height(Length::Fixed(1.0))
        .style(|_theme| {
            use container::Style;

            Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.8, 0.8))),
                ..Style::default()
            }
        })
}

pub fn radio_with_border<'a, M, T>(
    text: &'a str,
    to: T,
    selected: Option<T>,
    on_click: impl FnOnce(T) -> M
) -> Container<'a, M> 
where 
    M: Clone + 'a,
    T: Copy + Eq
{
    radio(
        text,
        to,
        selected,
        on_click,
    )
    .pad(*LENGTH_UNIT)
    .style(move |_theme| {
        use container::Style;

        let light_grey_color = Color::from_rgb(0.85, 0.85, 0.85);

        let border = Border {
            color: light_grey_color,
            width: 1.0,
            radius: Radius::new(*LENGTH_UNIT / 2.0),
        };

        Style {
            border,
            ..Style::default()
        }
    })
    .pad(*LENGTH_UNIT)
}


