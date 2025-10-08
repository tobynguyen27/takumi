use takumi::layout::{
  node::ContainerNode,
  style::{
    Color, ColorInput,
    LengthUnit::{Percentage, Px},
    StyleBuilder,
  },
};

mod test_utils;
use test_utils::run_style_width_test;

#[test]
fn test_style_width() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Percentage(100.0))
        .height(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_width.png");
}

#[test]
fn test_style_height() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Percentage(100.0))
        .height(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_height.png");
}

#[test]
fn test_style_min_width() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .min_width(Px(50.0))
        .width(Percentage(100.0))
        .height(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_min_width.png");
}

#[test]
fn test_style_min_height() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .min_height(Px(50.0))
        .height(Percentage(100.0))
        .width(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_min_height.png");
}

#[test]
fn test_style_max_width() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .max_width(Px(100.0))
        .width(Percentage(100.0))
        .height(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_max_width.png");
}

#[test]
fn test_style_max_height() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .max_height(Px(100.0))
        .height(Percentage(100.0))
        .width(Percentage(100.0))
        .background_color(ColorInput::Value(Color::white()))
        .build()
        .unwrap(),
    ),
    children: None,
  };

  run_style_width_test(container.into(), "tests/fixtures/style_max_height.png");
}
