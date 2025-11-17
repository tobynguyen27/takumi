import { docs } from ".source/server";
import { loader } from "fumadocs-core/source";
import defaultAttributes from "lucide-react/dist/esm/defaultAttributes";
import { __iconNode as arrowBigRightIconNode } from "lucide-react/dist/esm/icons/arrow-big-right";
import { __iconNode as axeIconNode } from "lucide-react/dist/esm/icons/axe";
import { __iconNode as bookIconNode } from "lucide-react/dist/esm/icons/book";
import { __iconNode as bookMarkedIconNode } from "lucide-react/dist/esm/icons/book-marked";
import { __iconNode as brainIconNode } from "lucide-react/dist/esm/icons/brain";
import { __iconNode as bugIconNode } from "lucide-react/dist/esm/icons/bug";
import { __iconNode as handIconNode } from "lucide-react/dist/esm/icons/hand";
import { __iconNode as leafIconNode } from "lucide-react/dist/esm/icons/leaf";
import { __iconNode as toyBrickIconNode } from "lucide-react/dist/esm/icons/toy-brick";
import { createElement } from "react";

const iconProps = {
  ...defaultAttributes,
  color: "currentColor",
  strokeWidth: 2,
  width: 24,
  height: 24,
};

const icons = {
  Leaf: leafIconNode,
  Brain: brainIconNode,
  Book: bookIconNode,
  ToyBrick: toyBrickIconNode,
  Axe: axeIconNode,
  ArrowBigRight: arrowBigRightIconNode,
  Hand: handIconNode,
  Bug: bugIconNode,
  BookMarked: bookMarkedIconNode,
};

export const source = loader({
  icon(icon) {
    if (icon && icon in icons) {
      return createElement(
        "svg",
        iconProps,
        icons[icon as keyof typeof icons].map(([tag, attrs]) =>
          createElement(tag, attrs),
        ),
      );
    }
  },
  source: docs.toFumadocsSource(),
  baseUrl: "/docs",
});
