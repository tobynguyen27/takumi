import { describe, expect, test } from "bun:test";
import type { TwConfig } from "twrnc";
import { createTailwindFn } from "../../src/jsx/create-tailwind-fn";
import { fromJsx } from "../../src/jsx/jsx";
import type { ContainerNode, TextNode } from "../../src/types";

describe("createTailwindFn", () => {
  test("creates a tailwind function", () => {
    const tw = createTailwindFn();
    expect(typeof tw).toBe("function");
  });

  test("tailwind function processes basic classes", () => {
    const tw = createTailwindFn();
    const styles = tw`text-red-500 bg-blue-100 p-4`;

    expect(styles).toEqual(
      expect.objectContaining({
        color: "#ef4444", // text-red-500
        backgroundColor: "#dbeafe", // bg-blue-100
        paddingBottom: 16, // p-4
        paddingLeft: 16,
        paddingRight: 16,
        paddingTop: 16,
      }),
    );
  });

  test("includes default shadow utilities", () => {
    const tw = createTailwindFn();

    const shadowStyles = tw`shadow`;
    expect(shadowStyles).toHaveProperty("boxShadow");

    const shadowMdStyles = tw`shadow-md`;
    expect(shadowMdStyles).toHaveProperty("boxShadow");

    const shadowNoneStyles = tw`shadow-none`;
    expect(shadowNoneStyles).toHaveProperty("boxShadow", "0 0 #0000");
  });

  test("merges custom config with defaults", () => {
    const customConfig: TwConfig = {
      theme: {
        extend: {
          colors: {
            custom: "#123456",
          },
        },
      },
    };

    const tw = createTailwindFn(customConfig);
    const styles = tw`text-custom`;

    expect(styles).toHaveProperty("color", "#123456");
  });

  test("preserves existing plugins when adding custom config", () => {
    const customConfig: TwConfig = {
      plugins: [
        {
          handler({ addUtilities }) {
            addUtilities({
              "custom-utility": { fontSize: "24px" },
            });
          },
        },
      ],
    };

    const tw = createTailwindFn(customConfig);

    // Should have both custom utility and default shadows
    const customStyles = tw`custom-utility`;
    expect(customStyles).toHaveProperty("fontSize", "24px");

    const shadowStyles = tw`shadow`;
    expect(shadowStyles).toHaveProperty("boxShadow");
  });

  test("handles empty config", () => {
    const tw = createTailwindFn({});
    const styles = tw`text-center`;
    expect(styles).toHaveProperty("textAlign", "center");
  });

  test("handles undefined config", () => {
    const tw = createTailwindFn(undefined);
    const styles = tw`text-center`;
    expect(styles).toHaveProperty("textAlign", "center");
  });
});

describe("Tailwind JSX Integration", () => {
  test("processes tw prop on JSX elements", async () => {
    const result = await fromJsx(
      <div tw="text-red-500 bg-blue-100">Hello</div>,
      { tailwindFn: createTailwindFn() },
    );

    expect(result).toEqual({
      type: "text",
      text: "Hello",
      style: expect.objectContaining({
        color: "#ef4444", // text-red-500
        backgroundColor: "#dbeafe", // bg-blue-100
      }),
    } satisfies TextNode);
  });

  test("combines tw prop with style prop", async () => {
    const result = await fromJsx(
      <div
        tw="text-red-500"
        style={{ fontSize: 20, backgroundColor: "yellow" }}
      >
        Hello
      </div>,
      { tailwindFn: createTailwindFn() },
    );

    expect(result).toEqual({
      type: "text",
      text: "Hello",
      style: expect.objectContaining({
        color: "#ef4444", // from tw prop
        fontSize: 20, // from style prop
        backgroundColor: "yellow", // style prop overrides tailwind
      }),
    } satisfies TextNode);
  });

  test("handles multiple tailwind classes", async () => {
    const result = await fromJsx(
      <div tw="flex items-center justify-between p-4 bg-gray-100 text-lg font-bold">
        Content
      </div>,
      { tailwindFn: createTailwindFn() },
    );

    expect(result).toEqual({
      type: "text",
      text: "Content",
      style: expect.objectContaining({
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        paddingBottom: 16,
        paddingLeft: 16,
        paddingRight: 16,
        paddingTop: 16,
        backgroundColor: "#f3f4f6",
        fontSize: 18,
        fontWeight: "bold",
      }),
    } satisfies TextNode);
  });

  test("ignores tw prop when no tailwindFn provided", async () => {
    const result = await fromJsx(<div tw="text-red-500">Hello</div>);

    expect(result).toEqual({
      type: "text",
      text: "Hello",
    } satisfies TextNode);
  });

  test("handles empty tw prop", async () => {
    const result = await fromJsx(<div tw="">Hello</div>, {
      tailwindFn: createTailwindFn(),
    });

    expect(result).toEqual({
      type: "text",
      text: "Hello",
    } satisfies TextNode);
  });

  test("handles tw prop with invalid classes gracefully", async () => {
    const result = await fromJsx(
      <div tw="invalid-class text-red-500">Hello</div>,
      { tailwindFn: createTailwindFn() },
    );

    // Should still apply valid classes
    expect(result).toEqual({
      type: "text",
      text: "Hello",
      style: expect.objectContaining({
        color: "#ef4444", // text-red-500 should work
      }),
    } satisfies TextNode);
  });

  test("works with custom tailwind config in JSX", async () => {
    const customConfig: TwConfig = {
      theme: {
        extend: {
          colors: {
            brand: "#ff6b6b",
          },
        },
      },
    };

    const result = await fromJsx(<div tw="text-brand bg-white">Branded</div>, {
      tailwindFn: createTailwindFn(customConfig),
    });

    expect(result).toEqual({
      type: "text",
      text: "Branded",
      style: expect.objectContaining({
        color: "#ff6b6b",
        backgroundColor: "#fff",
      }),
    } satisfies TextNode);
  });

  test("applies tailwind classes to container elements", async () => {
    const result = await fromJsx(
      <div tw="p-4 bg-blue-500">
        <span tw="text-white font-bold">Nested</span>
      </div>,
      { tailwindFn: createTailwindFn() },
    );

    expect(result).toEqual({
      type: "container",
      children: [
        {
          type: "text",
          text: "Nested",
          style: expect.objectContaining({
            color: "#fff",
            fontWeight: "bold",
          }),
        },
      ],
      style: expect.objectContaining({
        paddingBottom: 16,
        paddingLeft: 16,
        paddingRight: 16,
        paddingTop: 16,
        backgroundColor: "#3b82f6",
      }),
    } satisfies ContainerNode);
  });

  test("handles shadow utilities in JSX", async () => {
    const result = await fromJsx(
      <div tw="shadow-md p-4 bg-white">Shadowed</div>,
      { tailwindFn: createTailwindFn() },
    );

    expect(result).toEqual({
      type: "text",
      text: "Shadowed",
      style: expect.objectContaining({
        boxShadow:
          "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
        paddingBottom: 16,
        paddingLeft: 16,
        paddingRight: 16,
        paddingTop: 16,
        backgroundColor: "#fff",
      }),
    } satisfies TextNode);
  });

  test("preserves style presets when using tailwind", async () => {
    const result = await fromJsx(<h1 tw="text-blue-600">Title</h1>, {
      tailwindFn: createTailwindFn(),
    });

    expect(result).toEqual({
      type: "text",
      text: "Title",
      style: expect.objectContaining({
        color: "#2563eb", // from tailwind
        fontSize: "2em", // from h1 preset
        fontWeight: "bold", // from h1 preset
        marginBottom: "0.67em", // from h1 preset
        marginLeft: 0,
        marginRight: 0,
        marginTop: "0.67em",
        display: "block",
      }),
    } satisfies TextNode);
  });
});
