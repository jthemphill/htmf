import { spawn } from "child_process";
import { task } from "hereby";

export const clean = task({
  name: "clean",
  run: async () => {
    await Promise.all([
      exec("rm", ["-rf", "wasm/pkg"]),
      exec("bun", ["--recursive", "--parallel", "exec", "rm", "-rf", "dist"]),
    ]);
  },
});

export const install_cargo = task({
  name: "install_cargo",
  run: async () => {
    try {
      await exec("command", ["-v", "cargo"]);
    } catch (err) {
      if (err instanceof ExecError && err.exitCode === 1) {
        console.log(
          "Cargo not found. Please install Rust from https://rustup.rs/"
        );
        process.exit(1);
      }
    }
  },
});

export const test_rust = task({
  name: "test_rust",
  run: async () => {
    await exec("cargo", ["test"]);
  },
  dependencies: [install_cargo],
});

export const install_wasm_pack = task({
  name: "install_wasm_pack",
  run: async () => {
    await exec("cargo", ["install", "wasm-pack"]);
  },
  dependencies: [install_cargo],
});

export const build_wasm_st = task({
  name: "build_wasm_st",
  run: async () => {
    await exec("wasm-pack", [
      "build",
      "wasm",
      "--target",
      "web",
      "--profiling",
    ]);
  },
  dependencies: [install_wasm_pack],
});

export const install = task({
  name: "install",
  run: async () => {
    await exec("bun", ["install"]);
  },
  dependencies: [build_wasm_st],
});

export const playwright_install = task({
  name: "playwright_install",
  run: async () => {
    await exec("bunx", ["playwright", "install", "--with-deps"], {
      cwd: "www",
    });
  },
  dependencies: [],
});

export const dev = task({
  name: "dev",
  run: async () => {
    await exec("bun", ["run", "dev"], { cwd: "www" });
  },
  dependencies: [install],
});

export const build_www = task({
  name: "build_www",
  run: async () => {
    await exec("bun", ["run", "build"], { cwd: "www" });
  },
  dependencies: [install],
});

export const lint_www = task({
  name: "lint_www",
  run: async () => {
    await exec("bun", ["run", "lint"], { cwd: "www" });
  },
  dependencies: [install],
});

export const typecheck_training = task({
  name: "typecheck_training",
  run: async () => {
    await exec("pnpm", ["run", "typecheck"], { cwd: "training" });
  },
  dependencies: [install],
});

export const generate_data = task({
  name: "generate_data",
  run: async () => {
    await exec("pnpm", ["run", "generate-data"], { cwd: "training" });
  },
  dependencies: [test_rust],
});

export const train = task({
  name: "train",
  run: async () => {
    await exec("pnpm", ["run", "train"], { cwd: "training" });
  },
  dependencies: [generate_data],
});

export const typecheck_www = task({
  name: "typecheck_www",
  run: async () => {
    await exec("bun", ["run", "typecheck"], { cwd: "www" });
  },
  dependencies: [install],
});

export const typecheck = task({
  name: "typecheck",
  dependencies: [typecheck_training, typecheck_www],
});

export const build = task({
  name: "build",
  dependencies: [build_www, lint_www, typecheck_www],
});

export const preview = task({
  name: "preview",
  run: async () => {
    await exec("bun", ["run", "preview"], { cwd: "www" });
  },
  dependencies: [build],
});

export const test_www = task({
  name: "test_www",
  run: async () => {
    await exec("bun", ["run", "test:headless"], { cwd: "www" });
  },
  dependencies: [install, playwright_install],
});

export const test = task({
  name: "test",
  dependencies: [test_rust, lint_www, typecheck, test_www],
});

export const deploy = task({
  name: "deploy",
  run: async () => {
    await exec("bun", ["run", "deploy:pages"], { cwd: "www" });
  },
  dependencies: [build, install, test],
});

/**
 * Executes the provided command once with the supplied arguments.
 * @param {string} cmd
 * @param {string[]} args
 * @param {ExecOptions} [options]
 *
 * @typedef ExecOptions
 * @property {string} [cwd]
 * @property {boolean} [ignoreExitCode]
 * @property {boolean} [hidePrompt]
 * @property {boolean} [waitForExit=true]
 * @property {boolean} [ignoreStdout]
 */
export async function exec(cmd, args, options = {}) {
  return /**@type {Promise<{exitCode?: number}>}*/ (
    new Promise((resolve, reject) => {
      const { cwd, ignoreExitCode, waitForExit = true, ignoreStdout } = options;

      if (!options.hidePrompt) console.log(`> ${cmd} ${args.join(" ")}`);
      const proc = spawn(cmd, args, {
        cwd,
        stdio: waitForExit
          ? ignoreStdout
            ? ["inherit", "ignore", "inherit"]
            : "inherit"
          : "ignore",
        detached: !waitForExit,
      });
      if (waitForExit) {
        proc.on("exit", (exitCode) => {
          if (exitCode === 0 || ignoreExitCode) {
            resolve({ exitCode: exitCode ?? undefined });
          } else {
            const reason = new ExecError(exitCode);
            reject(reason);
          }
        });
        proc.on("error", (error) => {
          reject(error);
        });
      } else {
        proc.unref();
        resolve({ exitCode: undefined });
      }
    })
  );
}

export class ExecError extends Error {
  exitCode;

  /**
   * @param {number | null} exitCode
   * @param {string} message
   */
  constructor(exitCode, message = `Process exited with code: ${exitCode}`) {
    super(message);
    this.exitCode = exitCode;
  }
}
