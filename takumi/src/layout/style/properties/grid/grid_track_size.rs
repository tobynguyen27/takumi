use cssparser::Parser;
use serde::{Deserialize, Serialize};
use taffy::{MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction};
use ts_rs::TS;

use crate::{
  layout::style::{FromCss, GridLengthUnit, GridMinMaxSize, ParseResult},
  rendering::RenderContext,
};

/// A wrapper around a list of `GridTrackSize` that can also be parsed from a CSS string.
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(try_from = "GridTrackSizesValue")]
#[ts(as = "GridTrackSizesValue")]
pub struct GridTrackSizes(pub Vec<GridTrackSize>);

/// Serializable input for `GridTrackSizes` that accepts either a list of
/// pre-parsed `GridTrackSize` values or a CSS string to parse.
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub(crate) enum GridTrackSizesValue {
  /// Explicit list of track sizes.
  Components(Vec<GridTrackSize>),
  /// CSS value to parse (e.g. "minmax(10px, 1fr) 2fr").
  Css(String),
}

impl<'i> FromCss<'i> for GridTrackSizes {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut components: Vec<GridTrackSize> = Vec::new();
    while let Ok(size) = GridTrackSize::from_css(input) {
      components.push(size);
    }

    Ok(GridTrackSizes(components))
  }
}

impl TryFrom<GridTrackSizesValue> for GridTrackSizes {
  type Error = String;

  fn try_from(value: GridTrackSizesValue) -> Result<Self, Self::Error> {
    match value {
      GridTrackSizesValue::Components(components) => Ok(GridTrackSizes(components)),
      GridTrackSizesValue::Css(css) => GridTrackSizes::from_str(&css).map_err(|e| e.to_string()),
    }
  }
}

/// Represents a grid track size
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub enum GridTrackSize {
  /// A minmax() track size
  MinMax(GridMinMaxSize),
  /// A fixed track size
  Fixed(GridLengthUnit),
}

impl From<GridLengthUnit> for GridTrackSize {
  fn from(length: GridLengthUnit) -> Self {
    Self::Fixed(length)
  }
}

impl GridTrackSize {
  /// Converts the grid track size to a non-repeated track sizing function.
  pub fn to_min_max(&self, context: &RenderContext) -> TrackSizingFunction {
    match self {
      // SAFETY: The compact length is a valid track sizing function.
      Self::Fixed(size) => unsafe {
        TrackSizingFunction {
          min: MinTrackSizingFunction::from_raw(size.to_compact_length(context)),
          max: MaxTrackSizingFunction::from_raw(size.to_compact_length(context)),
        }
      },
      Self::MinMax(min_max) => unsafe {
        TrackSizingFunction {
          min: MinTrackSizingFunction::from_raw(min_max.min.to_compact_length(context)),
          max: MaxTrackSizingFunction::from_raw(min_max.max.to_compact_length(context)),
        }
      },
    }
  }
}

impl<'i> FromCss<'i> for GridTrackSize {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    if input
      .try_parse(|input| input.expect_function_matching("minmax"))
      .is_ok()
    {
      return input.parse_nested_block(|input| {
        let min = GridLengthUnit::from_css(input)?;
        input.expect_comma()?;
        let max = GridLengthUnit::from_css(input)?;
        Ok(GridTrackSize::MinMax(GridMinMaxSize { min, max }))
      });
    }

    let length = GridLengthUnit::from_css(input)?;
    Ok(GridTrackSize::Fixed(length))
  }
}

#[cfg(test)]
mod tests {
  use crate::layout::style::LengthUnit;

  use super::*;

  #[test]
  fn test_parse_minmax_and_track_size() {
    let minmax = GridTrackSize::from_str("minmax(10px, 1fr)").unwrap();
    match minmax {
      GridTrackSize::MinMax(m) => {
        assert_eq!(m.min, GridLengthUnit::Unit(LengthUnit::Px(10.0)));
        assert_eq!(m.max, GridLengthUnit::Fr(1.0));
      }
      _ => panic!("expected minmax"),
    }

    let fixed = GridTrackSize::from_str("2fr").unwrap();
    assert_eq!(fixed, GridTrackSize::Fixed(GridLengthUnit::Fr(2.0)));
  }
}
