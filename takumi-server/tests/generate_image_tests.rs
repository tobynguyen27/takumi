use axum::extract::Query;
use takumi::{
  GlobalContext,
  layout::{
    node::{ContainerNode, NodeKind},
    style::{LengthUnit::Px, StyleBuilder},
  },
};

use takumi_server::{GenerateImageQuery, args::Args, create_state, generate_image_handler};

#[tokio::test]
async fn test_generate_image_handler() {
  let node: NodeKind = ContainerNode {
    style: Some(
      StyleBuilder::default()
        .width(Px(100.0))
        .height(Px(100.0))
        .build()
        .unwrap(),
    ),
    children: None,
  }
  .into();

  let state = create_state(Args::default(), GlobalContext::default());
  let response = generate_image_handler(
    Query(GenerateImageQuery {
      format: None,
      quality: None,
      payload: serde_json::to_string(&node).unwrap(),
      draw_debug_border: Some(false),
      width: 1200,
      height: 630,
    }),
    state,
  )
  .await
  .unwrap();
  assert_eq!(response.status(), 200);
}
