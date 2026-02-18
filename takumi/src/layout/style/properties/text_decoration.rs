use bitflags::bitflags;
use cssparser::{Parser, Token};

use crate::{
  layout::style::{CssToken, FromCss, MakeComputed, ParseResult, properties::ColorInput},
  rendering::Sizing,
};

bitflags! {
  /// Represents a collection of text decoration lines.
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
  pub struct TextDecorationLines: u8 {
    /// Underline text decoration.
    const UNDERLINE = 0b001;
    /// Line-through text decoration.
    const LINE_THROUGH = 0b010;
    /// Overline text decoration.
    const OVERLINE = 0b100;
  }
}

fn parse_text_decoration_line<'i>(
  input: &mut Parser<'i, '_>,
) -> ParseResult<'i, TextDecorationLines> {
  let location = input.current_source_location();
  let token = input.next()?;

  if let Token::Ident(ident) = token {
    if ident.eq_ignore_ascii_case("underline") {
      return Ok(TextDecorationLines::UNDERLINE);
    }

    if ident.eq_ignore_ascii_case("line-through") {
      return Ok(TextDecorationLines::LINE_THROUGH);
    }

    if ident.eq_ignore_ascii_case("overline") {
      return Ok(TextDecorationLines::OVERLINE);
    }
  }

  Err(TextDecorationLines::unexpected_token_error(location, token))
}

impl<'i> FromCss<'i> for TextDecorationLines {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut lines = TextDecorationLines::empty();

    while !input.is_exhausted() {
      lines |= parse_text_decoration_line(input)?;
    }

    Ok(lines)
  }

  fn valid_tokens() -> &'static [CssToken] {
    &[
      CssToken::Keyword("underline"),
      CssToken::Keyword("line-through"),
      CssToken::Keyword("overline"),
    ]
  }
}

impl MakeComputed for TextDecorationLines {}

/// Represents text decoration style options (currently only solid is supported).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDecorationStyle {
  /// Solid text decoration style.
  Solid,
}

/// Parsed `text-decoration` value.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextDecoration {
  /// Text decoration line style.
  pub line: TextDecorationLines,
  /// Text decoration style (currently only solid is supported).
  pub style: Option<TextDecorationStyle>,
  /// Optional text decoration color.
  pub color: Option<ColorInput>,
}

impl MakeComputed for TextDecorationStyle {}

impl MakeComputed for TextDecoration {
  fn make_computed(&mut self, sizing: &Sizing) {
    self.color.make_computed(sizing);
  }
}

impl<'i> FromCss<'i> for TextDecoration {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut line = TextDecorationLines::empty();
    let mut style = None;
    let mut color = None;

    loop {
      if let Ok(value) = input.try_parse(parse_text_decoration_line) {
        line |= value;
        continue;
      }

      if let Ok(value) = input.try_parse(TextDecorationStyle::from_css) {
        style = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(ColorInput::from_css) {
        color = Some(value);
        continue;
      }

      if input.is_exhausted() {
        break;
      }

      return Err(Self::unexpected_token_error(
        input.current_source_location(),
        input.next()?,
      ));
    }

    Ok(TextDecoration { line, style, color })
  }

  fn valid_tokens() -> &'static [CssToken] {
    &[
      CssToken::Keyword("underline"),
      CssToken::Keyword("line-through"),
      CssToken::Keyword("overline"),
      CssToken::Keyword("solid"),
      CssToken::Token("color"),
    ]
  }
}

impl<'i> FromCss<'i> for TextDecorationStyle {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let token = input.next()?;

    if let Token::Ident(ident) = token
      && ident.eq_ignore_ascii_case("solid")
    {
      return Ok(TextDecorationStyle::Solid);
    }

    Err(Self::unexpected_token_error(location, token))
  }

  fn valid_tokens() -> &'static [CssToken] {
    &[CssToken::Keyword("solid")]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::layout::style::properties::Color;

  #[test]
  fn test_parse_text_decoration_underline() {
    assert_eq!(
      TextDecoration::from_str("underline"),
      Ok(TextDecoration {
        line: TextDecorationLines::UNDERLINE,
        style: None,
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_text_decoration_line_through() {
    assert_eq!(
      TextDecoration::from_str("line-through"),
      Ok(TextDecoration {
        line: TextDecorationLines::LINE_THROUGH,
        style: None,
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_text_decoration_underline_solid() {
    assert_eq!(
      TextDecoration::from_str("underline solid"),
      Ok(TextDecoration {
        line: TextDecorationLines::UNDERLINE,
        style: Some(TextDecorationStyle::Solid),
        color: None,
      })
    );
  }

  #[test]
  fn test_parse_text_decoration_line_through_solid_red() {
    assert_eq!(
      TextDecoration::from_str("line-through solid red"),
      Ok(TextDecoration {
        line: TextDecorationLines::LINE_THROUGH,
        style: Some(TextDecorationStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_text_decoration_multiple_lines() {
    assert_eq!(
      TextDecoration::from_str("underline line-through solid red"),
      Ok(TextDecoration {
        line: TextDecorationLines::UNDERLINE | TextDecorationLines::LINE_THROUGH,
        style: Some(TextDecorationStyle::Solid),
        color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
      })
    );
  }

  #[test]
  fn test_parse_text_decoration_invalid() {
    let result = TextDecoration::from_str("invalid");
    assert!(result.is_err());
  }
}
