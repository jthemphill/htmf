import React from "react";
import { beforeEach, describe, expect, test } from "vitest";
import { cleanup, render } from "vitest-browser-react";
import { page, userEvent } from "@vitest/browser/context";

import App from "./App";

describe("App", () => {
  beforeEach(() => {
    cleanup();
    render(<App />);
  });

  test("has clickable hexes", async () => {
    await expect.element(page.getByTestId("board")).toBeInTheDocument();

    const getClickableHexes = () =>
      page
        .getByRole("button")
        .elements()
        .filter(($hexElement) => $hexElement.ariaDisabled === "false");

    await expect.poll(getClickableHexes).toHaveLength(30);
    const clickableHexes = getClickableHexes();
    const clickableHexIdx = Math.floor(Math.random() * clickableHexes.length);
    const chosenHex = clickableHexes[clickableHexIdx];
    if (chosenHex === undefined) {
      throw new Error("No clickable hexes found");
    }
    await expect.element(chosenHex).toBeInTheDocument();
    await expect.element(chosenHex).toHaveRole("button");
    await userEvent.click(chosenHex);
  });
});
