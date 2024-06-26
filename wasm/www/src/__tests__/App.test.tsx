import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, test, expect, beforeEach } from "vitest";

import { NUM_CELLS } from "../constants";
import App from "../App";
import { type BotWorker } from "../WorkerProtocol";
import { type WorkerRequest, type WorkerResponse } from "../WorkerProtocol";
import Bot from "../Bot";
import htmfWasmInit from "../../../pkg/htmf_wasm";

import fs from "fs";
import path from "node:path";

class MockWorker implements BotWorker {
  postMessage: (request: WorkerRequest) => void;
  onmessage: (this: Worker, ev: MessageEvent<WorkerResponse>) => void;
  onmessageerror: null;

  constructor() {
    this.postMessage = () => {};
    this.onmessage = () => {};
    this.onmessageerror = null;
  }

  terminate(): void {}
  addEventListener(): void {}
  removeEventListener(): void {}
  onerror(): void {}
  dispatchEvent(): boolean {
    return false;
  }
}

describe("App", async () => {
  const wasmFile = await fs.promises.readFile(
    path.resolve(__dirname, "../../../pkg/htmf_wasm_bg.wasm"),
  );
  beforeEach(async () => {
    const cleanupFuncs: Array<() => void> = [];

    try {
      const mockWorker = new MockWorker();
      cleanupFuncs.push(() => {
        mockWorker.terminate();
      });

      const wasmInternals = await htmfWasmInit(wasmFile.buffer);
      const bot = new Bot(wasmInternals, (response: WorkerResponse) => {
        if (mockWorker.onmessage !== null) {
          mockWorker.onmessage(
            new MessageEvent<WorkerResponse>("message", { data: response }),
          );
        } else {
          console.error(
            "Mock WebWorker's onmessage property hasn't been initialized yet.",
          );
        }
      });
      cleanupFuncs.push(() => {
        bot.free();
      });

      mockWorker.postMessage = (request: WorkerRequest) => {
        bot.onMessage(request);
      };
      render(<App worker={mockWorker} />);
    } catch (e) {
      for (const cleanupFunc of cleanupFuncs.reverse()) {
        cleanupFunc();
      }
      throw e;
    }
    return async () => {
      for (const cleanupFunc of cleanupFuncs.reverse()) {
        cleanupFunc();
      }
    };
  });

  test.concurrent("renders a game board", async () => {
    const buttons = await screen.findAllByRole("button");
    expect(
      buttons.filter((btn) => btn.classList.contains("cell")).length,
    ).toEqual(NUM_CELLS);
  });
});
