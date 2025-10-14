import path from "node:path";
import { fileURLToPath } from "node:url";

import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tseslint from "typescript-eslint";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * @type {ReturnType<typeof defineConfig>}
 */
const config = defineConfig([
  globalIgnores(["dist/**/*"]),
  {
    languageOptions: {
      globals: {
        ...globals.browser,
      },

      ecmaVersion: "latest",
      sourceType: "module",

      parserOptions: {
        projectService: true,
        tsconfigRootDir: __dirname,
      },
    },
  },
  js.configs.recommended,
  reactHooks.configs.flat.recommended,
  reactRefresh.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
]);

export default config;
