use cssparser::Parser;

use crate::layout::style::*;

/// Parsed `background` shorthand value.
///
/// The background shorthand allows setting multiple background properties at once:
/// - color
/// - image
/// - position (optionally followed by / size)
/// - repeat
/// - clip
///
/// Example: `background: red url(image.png) center/cover no-repeat border-box`
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Background {
  /// Background color.
  pub color: Option<ColorInput<false>>,
  /// Background image.
  pub image: Option<BackgroundImages>,
  /// Background position.
  pub position: Option<BackgroundPositions>,
  /// Background size.
  pub size: Option<BackgroundSizes>,
  /// Background repeat.
  pub repeat: Option<BackgroundRepeats>,
  /// Background clip.
  pub clip: Option<BackgroundClip>,
}

impl<'i> FromCss<'i> for Background {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut color = None;
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;
    let mut clip = None;

    loop {
      if input.is_exhausted() {
        break;
      }

      // Try to parse background-color
      if color.is_none()
        && let Ok(value) = input.try_parse(ColorInput::from_css)
      {
        color = Some(value);
        continue;
      }

      // Try to parse background-position (and optionally background-size with /)
      if position.is_none()
        && let Ok(value) = input.try_parse(BackgroundPositions::from_css)
      {
        position = Some(value);

        size = input
          .try_parse(|input| {
            input.expect_delim('/')?;
            BackgroundSizes::from_css(input)
          })
          .ok();

        continue;
      }

      // Try to parse background-image
      if image.is_none()
        && let Ok(value) = input.try_parse(BackgroundImages::from_css)
      {
        image = Some(value);
        continue;
      }

      // Try to parse background-repeat
      if repeat.is_none()
        && let Ok(value) = input.try_parse(BackgroundRepeats::from_css)
      {
        repeat = Some(value);
        continue;
      }

      // Try to parse background-clip
      if clip.is_none()
        && let Ok(value) = input.try_parse(BackgroundClip::from_css)
      {
        clip = Some(value);
        continue;
      }

      // If we can't parse anything, it's an error
      return Err(input.new_error_for_next_token());
    }

    Ok(Background {
      color,
      image,
      position,
      size,
      repeat,
      clip,
    })
  }
}

#[cfg(test)]
mod tests {
  use smallvec::smallvec;

  use super::*;

  #[test]
  fn test_parse_background_color_only() {
    assert_eq!(
      Background::from_str("red"),
      Ok(Background {
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
        image: None,
        position: None,
        size: None,
        repeat: None,
        clip: None,
      })
    );
  }

  #[test]
  fn test_parse_background_color_and_clip() {
    assert_eq!(
      Background::from_str("red border-box"),
      Ok(Background {
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
        image: None,
        position: None,
        size: None,
        repeat: None,
        clip: Some(BackgroundClip::BorderBox),
      })
    );
  }

  #[test]
  fn test_parse_background_with_position_and_size() {
    assert_eq!(
      Background::from_str("center/cover"),
      Ok(Background {
        color: None,
        image: None,
        position: Some(BackgroundPositions(vec![BackgroundPosition::default()])),
        size: Some(BackgroundSizes(vec![BackgroundSize::Cover])),
        repeat: None,
        clip: None,
      })
    );
  }

  #[test]
  fn test_parse_background_full() {
    assert_eq!(
      Background::from_str("no-repeat center/80% url(../img/image.png)"),
      Ok(Background {
        color: None,
        image: Some(BackgroundImages(smallvec![BackgroundImage::Url(
          "../img/image.png".into()
        )])),
        position: Some(BackgroundPositions(vec![BackgroundPosition::default()])),
        size: Some(BackgroundSizes(vec![BackgroundSize::Explicit {
          width: LengthUnit::Percentage(80.0),
          height: LengthUnit::Auto,
        }])),
        repeat: Some(BackgroundRepeats(vec![BackgroundRepeat::no_repeat()])),
        clip: None,
      })
    );
  }

  #[test]
  fn test_parse_background_empty() {
    assert_eq!(Background::from_str(""), Ok(Background::default()));
  }

  #[test]
  fn test_parse_background_invalid() {
    assert!(Background::from_str("invalid-value").is_err());
  }
}
