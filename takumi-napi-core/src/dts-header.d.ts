export interface FontDetails {
  name?: string;
  data: Uint8Array | ArrayBuffer;
  weight?: number;
  style?:
    | "normal"
    | "italic"
    | "oblique"
    | `oblique ${number}deg`
    | (string & {});
}

export type Font = FontDetails | Uint8Array | ArrayBuffer;

export interface AnyNode {
  type: string;
  // biome-ignore lint/suspicious/noExplicitAny: for extensibility
  [key: string]: any;
}
