use cssparser::{Parser, ParserInput, Token, match_ignore_ascii_case};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ts_rs::TS;

use crate::layout::style::{Angle, FromCss, ParseResult, parse_length_percentage};

/// Represents a single CSS filter operation
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Filter {
  /// Brightness multiplier (1 = unchanged). Accepts number or percentage
  Brightness(f32),
  /// Contrast multiplier (1 = unchanged). Accepts number or percentage
  Contrast(f32),
  /// Grayscale amount (0..1). Accepts number or percentage
  Grayscale(f32),
  /// Hue rotation in degrees
  HueRotate(Angle),
  /// Invert amount (0..1). Accepts number or percentage
  Invert(f32),
}

/// A list of filters
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS)]
#[serde(untagged)]
pub(crate) enum FiltersValue {
  /// Structured set of filters
  #[ts(as = "Vec<Filter>")]
  Structured(SmallVec<[Filter; 4]>),
  /// Raw CSS string to be parsed
  Css(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TS, Default)]
#[ts(as = "FiltersValue")]
#[serde(try_from = "FiltersValue")]
/// A list of filter operations
pub struct Filters(pub SmallVec<[Filter; 4]>);

impl TryFrom<FiltersValue> for Filters {
  type Error = String;

  fn try_from(value: FiltersValue) -> Result<Self, Self::Error> {
    match value {
      FiltersValue::Structured(filters) => Ok(Filters(filters)),
      FiltersValue::Css(css) => {
        let mut input = ParserInput::new(&css);
        let mut parser = Parser::new(&mut input);

        let mut filters = SmallVec::new();

        while !parser.is_exhausted() {
          let filter = Filter::from_css(&mut parser).map_err(|e| e.to_string())?;
          filters.push(filter);
        }

        Ok(Filters(filters))
      }
    }
  }
}

impl<'i> FromCss<'i> for Filter {
  fn from_css(parser: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = parser.current_source_location();
    let token = parser.next()?;

    let Token::Function(function) = token else {
      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    };

    match_ignore_ascii_case! {function,
      "brightness" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Brightness(value))
      }),
      "contrast" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Contrast(value))
      }),
      "grayscale" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Grayscale(value))
      }),
      "hue-rotate" => parser.parse_nested_block(|input| {
        Ok(Filter::HueRotate(Angle::from_css(input)?))
      }),
      "invert" => parser.parse_nested_block(|input| {
        let value = parse_length_percentage(input)?;
        Ok(Filter::Invert(value))
      }),
      _ => Err(location.new_basic_unexpected_token_error(Token::Function(function.clone())).into()),
    }
  }
}
