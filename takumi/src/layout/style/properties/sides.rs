use cssparser::{Parser, ParserInput};
use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use taffy::Rect;
use ts_rs::TS;

use crate::layout::style::{FromCss, LengthUnit};

/// Represents the values for the four sides of a box (top, right, bottom, left).
#[derive(Debug, Clone, Copy, Serialize, TS, PartialEq)]
#[ts(as = "SidesValue<T>")]
pub struct Sides<T: TS + Copy>(pub [T; 4]);

/// Represents values that can be applied to all sides of an element.
///
/// This enum allows for flexible specification of values like padding, margin,
/// or border sizes using either a single value for all sides, separate values
/// for vertical/horizontal axes, or individual values for each side.
#[derive(Debug, Clone, Deserialize, Serialize, TS, PartialEq)]
#[serde(untagged)]
pub enum SidesValue<T> {
  /// CSS string representation
  Css(String),
  /// Individual values for each side (top, right, bottom, left)
  AllSides(T, T, T, T),
  /// Separate values for vertical and horizontal sides (vertical, horizontal)
  AxisSidesArray(T, T),
  /// Same value for all four sides
  SingleValue(T),
}

impl<T: TS + Copy + for<'i> FromCss<'i>> TryFrom<SidesValue<T>> for Sides<T> {
  type Error = String;

  fn try_from(value: SidesValue<T>) -> Result<Self, Self::Error> {
    match value {
      SidesValue::Css(string) => {
        let mut input = ParserInput::new(&string);
        let mut parser = Parser::new(&mut input);

        // Parse between 1 and 4 values of T using FromCss
        let first = T::from_css(&mut parser).map_err(|e| e.to_string())?;

        // Collect all values by parsing until we can't parse more
        let mut values = Vec::with_capacity(4);

        values.push(first);

        // Keep parsing values separated by whitespace
        loop {
          // Try to parse the next value
          match parser.try_parse(T::from_css) {
            Ok(next_value) => values.push(next_value),
            Err(_) => break,
          }

          // Don't allow more than 4 values
          if values.len() >= 4 {
            break;
          }
        }

        // Now create the sides based on how many values we got
        let sides = match values.len() {
          1 => Sides([values[0]; 4]),
          2 => Sides([values[0], values[1], values[0], values[1]]),
          3 => Sides([values[0], values[1], values[2], values[1]]),
          4 => Sides([values[0], values[1], values[2], values[3]]),
          _ => unreachable!(),
        };

        Ok(sides)
      }
      SidesValue::AllSides(top, right, bottom, left) => Ok(Sides([top, right, bottom, left])),
      SidesValue::AxisSidesArray(vertical, horizontal) => {
        Ok(Sides([vertical, horizontal, vertical, horizontal]))
      }
      SidesValue::SingleValue(value) => Ok(Sides([value; 4])),
    }
  }
}

impl<'de, T> Deserialize<'de> for Sides<T>
where
  T: TS + Copy + Deserialize<'de> + for<'i> FromCss<'i>,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let proxy = SidesValue::<T>::deserialize(deserializer)?;
    Sides::try_from(proxy).map_err(D::Error::custom)
  }
}

impl<T: Copy + TS> From<Sides<T>> for Rect<T> {
  fn from(value: Sides<T>) -> Self {
    Rect {
      top: value.0[0],
      right: value.0[1],
      bottom: value.0[2],
      left: value.0[3],
    }
  }
}

impl<T: TS + Default + Copy> Default for Sides<T> {
  fn default() -> Self {
    Self([T::default(); 4])
  }
}

impl<T: TS + Copy> From<T> for Sides<T> {
  fn from(value: T) -> Self {
    Self([value; 4])
  }
}

impl<T: Default> Default for SidesValue<T> {
  fn default() -> Self {
    Self::SingleValue(T::default())
  }
}

impl Sides<LengthUnit> {
  /// Creates a new zeroable Sides with [LengthUnit::zero].
  pub const fn zero() -> Self {
    Self([LengthUnit::zero(); 4])
  }

  /// Creates a new autoable Sides with [LengthUnit::Auto].
  pub const fn auto() -> Self {
    Self([LengthUnit::Auto; 4])
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::layout::style::LengthUnit;
  use serde_json;

  #[test]
  fn deserialize_single_number() {
    let json = "5";
    let sides: Sides<LengthUnit> = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(sides, Sides([LengthUnit::Px(5.0); 4]));
  }

  #[test]
  fn deserialize_axis_pair_numbers() {
    let json = "[10, 20]";
    let sides: Sides<LengthUnit> = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(
      sides,
      Sides([
        LengthUnit::Px(10.0),
        LengthUnit::Px(20.0),
        LengthUnit::Px(10.0),
        LengthUnit::Px(20.0)
      ])
    );
  }

  #[test]
  fn deserialize_all_sides_numbers() {
    let json = "[1, 2, 3, 4]";
    let sides: Sides<LengthUnit> = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(
      sides,
      Sides([
        LengthUnit::Px(1.0),
        LengthUnit::Px(2.0),
        LengthUnit::Px(3.0),
        LengthUnit::Px(4.0)
      ])
    );
  }

  #[test]
  fn deserialize_css_single_value() {
    let json = "\"10px\"";
    let sides: Sides<LengthUnit> = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(sides, Sides([LengthUnit::Px(10.0); 4]));
  }

  #[test]
  fn deserialize_css_multi_values() {
    let json = "\"1px 2px 3px 4px\"";
    let sides: Sides<LengthUnit> = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(
      sides,
      Sides([
        LengthUnit::Px(1.0),
        LengthUnit::Px(2.0),
        LengthUnit::Px(3.0),
        LengthUnit::Px(4.0)
      ])
    );
  }
}
