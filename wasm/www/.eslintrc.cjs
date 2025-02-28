module.exports = {
  root: true,
  env: { browser: true, es2024: true },
  extends: [
    "plugin:@typescript-eslint/strict-type-checked",
    "plugin:react-hooks/recommended",
  ],
  ignorePatterns: [
    "**/__tests__",
    ".eslintrc.cjs",
    "dist",
    "setup-vitest.ts",
    "vite.config.ts",
  ],
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    project: ["./tsconfig.json"],
    tsconfigRootDir: __dirname,
  },
  plugins: ["eslint-plugin-react-compiler", "react-refresh"],
  rules: {
    "react-compiler/react-compiler": "error",
  },
};
