/// Common-use style definitions here
use iced::{
    border::radius, color, font, widget::container, Background, Border, Color, Font, Theme,
};

pub fn porter_sans_inline_font(is_font_loaded: bool) -> Font {
    if is_font_loaded {
        Font {
            family: font::Family::Name("Porter Sans Block"),
            ..Default::default()
        }
    } else {
        Font {
            ..Default::default()
        }
    }
}

/// Widget: Container
///
/// BG Color: R 200 G 200 B 200 White
///
/// Border: Width 2.0, Radius 10
pub fn cont_w_2_10(_theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: Color::from_rgb8(200, 200, 200),
            width: 2.0,
            radius: radius(10),
        },
        ..Default::default()
    }
}

/// Widget: Container
///
/// BG Color: R 55 G 55 B 55 Gray
///
/// Border: Width 0.0, Radius 10
pub fn cont_gray_0_10(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(55, 55, 55))),
        border: Border {
            color: Color::from_rgb8(55, 55, 55),
            width: 0.0,
            radius: radius(10),
        },
        ..Default::default()
    }
}

/// Widget: Container
///
/// BG Color: HEX #25510F greenish dark
///
/// Border: Width 0.0, Radius 10
pub fn cont_25510f_0_10(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x25510f))),
        border: Border {
            color: color!(0x25510f),
            width: 0.0,
            radius: radius(10),
        },
        ..Default::default()
    }
}

/// Widget: Container
///
/// BG Color: HEX #532206 orangeish dark
///
/// Border: Width 0.0, Radius 10
pub fn cont_532206_0_10(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x532206))),
        border: Border {
            color: color!(0x532206),
            width: 0.0,
            radius: radius(10),
        },
        ..Default::default()
    }
}

/// Widget: Container
///
/// BG Color: HEX #510515 reddish dark
///
/// Border: Width 0.0, Radius 10
pub fn cont_510515_0_10(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x510515))),
        border: Border {
            color: color!(0x510515),
            width: 0.0,
            radius: radius(10),
        },
        ..Default::default()
    }
}
