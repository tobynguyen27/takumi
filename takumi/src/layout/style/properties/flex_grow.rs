use serde::{
  Deserialize, Deserializer, Serialize,
  de::{Error, Unexpected},
};
use serde_untagged::UntaggedEnumVisitor;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Copy, TS, PartialEq)]
#[ts(type = "number | string")]
/// Represents a flex grow value.
pub struct FlexGrow(pub f32);

impl<'de> Deserialize<'de> for FlexGrow {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    UntaggedEnumVisitor::new()
      .f32(|num| Ok(FlexGrow(num)))
      .f64(|num| Ok(FlexGrow(num as f32)))
      .string(|str| {
        Ok(FlexGrow(str.parse::<f32>().map_err(|_| {
          serde_untagged::de::Error::invalid_value(Unexpected::Str(str), &"a number")
        })?))
      })
      .deserialize(deserializer)
  }
}
