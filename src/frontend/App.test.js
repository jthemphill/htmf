import * as React from "react";
import { beforeEach, describe, expect, test } from "vitest";
import { cleanup, render } from "vitest-browser-react";
import { page, userEvent } from "vitest/browser";
import App from "./App";
function getRandom(choices) {
    const i = Math.floor(Math.random() * choices.length);
    return choices[i];
}
describe("App", () => {
    beforeEach(async () => {
        await cleanup();
        await render(React.createElement(App, null));
        await expect.element(page.getByTestId("board")).toBeInTheDocument();
    });
    const getClickableHexes = () => page
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
    const clickPauseButton = async () => {
        const pauseButton = page.getByRole("button", { name: "Pause" });
        await expect.element(pauseButton).toBeInTheDocument();
        await userEvent.click(pauseButton);
    };
    const clickResumeButton = async () => {
        const resumeButton = page.getByRole("button", { name: "Resume" });
        await expect.element(resumeButton).toBeInTheDocument();
        await userEvent.click(resumeButton);
    };
    const clickMoveNowButton = async () => {
        const moveNowButton = page.getByRole("button", { name: "Move now" });
        await expect.element(moveNowButton).toBeInTheDocument();
        await userEvent.click(moveNowButton);
    };
    const getBotPenguinCount = () => page.getByTestId("board").element().querySelectorAll(".penguin.bot").length;
    test("has clickable hexes", async () => {
        await expect.poll(getClickableHexes).toHaveLength(30);
        await clickRandomHex();
    });
    test("pause button becomes resume button", async () => {
        // Initially, button should say "Pause"
        await clickPauseButton();
        // After clicking, button should say "Resume"
        await clickResumeButton();
        // Button should say "Pause" again
        await clickPauseButton();
    });
    test("opponent moves after player moves", async () => {
        // Wait for initial state - 30 one-fish hexes are clickable
        await expect.poll(getClickableHexes).toHaveLength(30);
        // Place the human's first penguin
        await clickRandomHex();
        await clickMoveNowButton();
        // Wait for the human's penguin to be placed (29 clickable hexes remain)
        // Without pause, bot will also place two penguins, so we should see 27 hexes
        await expect.poll(getClickableHexes).toHaveLength(27);
        // Bot should have 2 penguins since it wasn't paused
        expect(getBotPenguinCount()).toBe(2);
    });
    test("pause then move", async () => {
        // Wait for initial state - 30 one-fish hexes are clickable
        await expect.poll(getClickableHexes).toHaveLength(30);
        // Place the human's first penguin
        await clickPauseButton();
        await clickRandomHex();
        await clickMoveNowButton();
        // Wait for the human's penguin to be placed (29 clickable hexes remain)
        // Without pause, bot will also place two penguins, so we should see 27 hexes
        await expect.poll(getClickableHexes).toHaveLength(27);
        // Bot should have 2 penguins since it wasn't paused
        expect(getBotPenguinCount()).toBe(2);
    });
});
