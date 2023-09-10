module.exports = {
    root: true,
    env: { "browser": true, "es2021": true },
    extends: [
        'standard-with-typescript',
        'plugin:react-hooks/recommended',
    ],
    ignorePatterns: ['dist', '.eslintrc.cjs'],
    parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        project: [
            './tsconfig.json',
            './tsconfig-vite.json',
            './tsconfig-tests.json'
        ],
        tsconfigRootDir: __dirname,
    },
    plugins: [
        'react-refresh',
    ],
    rules: {
    }
}
