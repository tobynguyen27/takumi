use takumi::layout::{
  node::{ContainerNode, TextNode},
  style::{Color, ColorInput, CssOption, LengthUnit::Px, Overflow, Overflows, StyleBuilder},
};

mod test_utils;
use test_utils::run_style_width_test;

#[test]
fn test_overflow_visible() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Px(200.0))
        .height(Px(200.0))
        .background_color(ColorInput::Value(Color::white()))
        .overflow(Overflows(Overflow::Visible, Overflow::Visible))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      ContainerNode {
        style: Some(
          StyleBuilder::default()
            .width(Px(300.0))
            .height(Px(300.0))
            .background_color(ColorInput::Value(Color([255, 0, 0, 255])))
            .build()
            .unwrap(),
        ),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/overflow_visible.png");
}

#[test]
fn test_overflow_hidden() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Px(200.0))
        .height(Px(200.0))
        .background_color(ColorInput::Value(Color::white()))
        .overflow(Overflows(Overflow::Hidden, Overflow::Hidden))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      ContainerNode {
        style: Some(
          StyleBuilder::default()
            .width(Px(300.0))
            .height(Px(300.0))
            .background_color(ColorInput::Value(Color([0, 255, 0, 255])))
            .build()
            .unwrap(),
        ),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/overflow_hidden.png");
}

#[test]
fn test_overflow_mixed_axes() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Px(200.0))
        .height(Px(200.0))
        .background_color(ColorInput::Value(Color::white()))
        .overflow_x(CssOption::some(Overflow::Visible))
        .overflow_y(CssOption::some(Overflow::Hidden))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      ContainerNode {
        style: Some(
          StyleBuilder::default()
            .width(Px(300.0))
            .height(Px(300.0))
            .background_color(ColorInput::Value(Color([255, 0, 255, 255])))
            .build()
            .unwrap(),
        ),
        children: None,
      }
      .into(),
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/overflow_mixed_axes.png");
}

#[test]
fn test_overflow_with_text() {
  let container = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Px(200.0))
        .height(Px(200.0))
        .background_color(ColorInput::Value(Color::white()))
        .overflow(Overflows(Overflow::Hidden, Overflow::Hidden))
        .build()
        .unwrap(),
    ),
    children: Some(vec![
      TextNode {
        text: "This is a very long text that should overflow the container and demonstrate how overflow hidden works with text content. The text should be clipped when it exceeds the container boundaries.".to_string(),
        style: Some(
          StyleBuilder::default()
            .font_size(CssOption::some(Px(16.0)))
            .color(ColorInput::Value(Color::black()))
            .build()
            .unwrap(),
        ),
      }.into()
    ]),
  };

  run_style_width_test(container.into(), "tests/fixtures/overflow_text.png");
}
