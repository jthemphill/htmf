import { spawn } from "child_process";
import { task } from "hereby";

export const clean = task({
  name: "clean",
  run: async () => {
    await Promise.all([
      exec("rm", ["-rf", "wasm/pkg"]),
      exec("pnpm", ["--recursive", "--parallel", "exec", "rm", "-rf", "dist"]),
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

export const install_wasm_pack = task({
  name: "install_wasm_pack",
  run: async () => {
    try {
      await exec("command", ["-v", "wasm-pack"]);
    } catch (err) {
      if (err instanceof ExecError && err.exitCode === 1) {
        await exec("cargo", ["install", "wasm-pack"]);
      }
    }
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
    await exec("pnpm", ["install"]);
  },
  dependencies: [build_wasm_st],
});

export const playwright_install = task({
  name: "playwright_install",
  run: async () => {
    await exec("pnpm", ["exec", "playwright", "install"], { cwd: "www" });
  },
  dependencies: [install],
});

export const dev = task({
  name: "dev",
  run: async () => {
    await exec("pnpm", ["run", "dev"], { cwd: "www" });
  },
  dependencies: [install],
});

export const build_www = task({
  name: "build_www",
  run: async () => {
    await exec("pnpm", ["run", "build"], { cwd: "www" });
  },
  dependencies: [install],
});

export const lint_www = task({
  name: "lint_www",
  run: async () => {
    await exec("pnpm", ["run", "lint"], { cwd: "www" });
  },
  dependencies: [install],
});

export const typecheck_www = task({
  name: "typecheck_www",
  run: async () => {
    await exec("pnpm", ["run", "typecheck"], { cwd: "www" });
  },
  dependencies: [install],
});

export const build = task({
  name: "build",
  dependencies: [build_www, lint_www, typecheck_www],
});

export const preview = task({
  name: "preview",
  run: async () => {
    await exec("pnpm", ["run", "preview"], { cwd: "www" });
  },
  dependencies: [build],
});

export const test = task({
  name: "test",
  run: async () => {
    await exec("pnpm", ["run", "test:headless"], { cwd: "www" });
  },
  dependencies: [install, playwright_install],
});

export const deploy = task({
  name: "deploy",
  run: async () => {
    await exec("pnpm", ["run", "deploy:pages"], { cwd: "www" });
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
