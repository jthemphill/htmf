import * as ort from "onnxruntime-web";
import type * as wasm from "htmf-wasm";

const MODEL_URL = "/models/htmf-policy.onnx";
const NUM_FEATURES = 8 * 60;

type Backend = "webgpu" | "wasm";

export default class PolicyModel {
  private session?: ort.InferenceSession;
  private backend?: Backend;
  private loadPromise?: Promise<void>;

  load(): Promise<void> {
    this.loadPromise ??= this.loadWithFallback();
    return this.loadPromise;
  }

  async applyRootPriors(game: wasm.Game): Promise<void> {
    await this.load();
    if (this.session === undefined || game.active_player() === undefined) {
      return;
    }

    const features = game.features_for_active_player();
    if (features.length !== NUM_FEATURES) {
      return;
    }

    const input = new ort.Tensor("float32", Float32Array.from(features), [
      1,
      NUM_FEATURES,
    ]);
    const outputs = await this.session.run({ features: input });
    const outputName = game.is_drafting() ? "drafting_policy" : "movement_policy";
    const output = outputs[outputName];
    if (output === undefined || !(output.data instanceof Float32Array)) {
      return;
    }

    game.apply_policy_logits(output.data);
  }

  getBackend(): Backend | undefined {
    return this.backend;
  }

  private async loadWithFallback(): Promise<void> {
    for (const backend of ["webgpu", "wasm"] satisfies Backend[]) {
      try {
        this.session = await ort.InferenceSession.create(MODEL_URL, {
          executionProviders: [backend],
          graphOptimizationLevel: "all",
        });
        this.backend = backend;
        return;
      } catch (err) {
        void err;
      }
    }
  }
}
