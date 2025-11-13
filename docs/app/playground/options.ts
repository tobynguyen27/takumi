declare type PlaygroundOptions = {
  /**
   * @description width of the render viewport.
   */
  width?: number;
  /**
   * @description height of the render viewport.
   */
  height?: number;
  /**
   * @description format to render.
   * @default png
   */
  format?: "png" | "jpeg" | "webp";
  /**
   * @description quality of jpeg format (0-100).
   * @default 75
   */
  quality?: number;
  /**
   * @description device pixel ratio.
   * @default 1.0
   */
  devicePixelRatio?: number;
};
