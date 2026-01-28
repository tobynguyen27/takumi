use takumi::layout::{
  node::TextNode,
  style::{Length::*, *},
};

use crate::test_utils::run_fixture_test;

#[test]
fn test_style_text_decoration() {
  let text = TextNode {
    preset: None,
    tw: None,
    style: Some(
      StyleBuilder::default()
        .width(Percentage(100.0))
        .text_align(TextAlign::Center)
        .background_color(ColorInput::Value(Color([240, 240, 240, 255])))
        .font_size(Some(Px(72.0)))
        .text_decoration(TextDecoration {
          line: [
            TextDecorationLine::Underline,
            TextDecorationLine::LineThrough,
            TextDecorationLine::Overline,
          ]
          .into(),
          style: None,
          color: Some(ColorInput::Value(Color([255, 0, 0, 255]))),
        })
        .build()
        .unwrap(),
    ),
    text: "Text Decoration with Underline, Line-Through, and Overline".to_string(),
  };

  run_fixture_test(text.into(), "style_text_decoration.webp");
}
