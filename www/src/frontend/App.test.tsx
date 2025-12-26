import * as React from "react";
import { beforeEach, describe, expect, test } from "vitest";
import { cleanup, render } from "vitest-browser-react";
import { page, userEvent } from "vitest/browser";

import App from "./App";

function getRandom<T>(choices: T[]): T | undefined {
  const i = Math.floor(Math.random() * choices.length);
  return choices[i];
}

describe("App", () => {
  beforeEach(async () => {
    await cleanup();
    await render(<App />);
    await expect.element(page.getByTestId("board")).toBeInTheDocument();
  });

  const getClickableHexes = () =>
    page
      .getByRole("button")
      .elements()
      .filter(($hexElement) => $hexElement.ariaDisabled === "false");

  const clickRandomHex = async () => {
    const chosenHex = getRandom(getClickableHexes());
    if (!chosenHex) {
      throw new Error("No clickable hexes found");
    }
    await expect.element(chosenHex).toBeInTheDocument();
    await expect.element(chosenHex).toHaveRole("button");
    await userEvent.click(chosenHex);
  };

  test("has clickable hexes", async () => {
    await expect.poll(getClickableHexes).toHaveLength(30);
    await clickRandomHex();
  });
});
