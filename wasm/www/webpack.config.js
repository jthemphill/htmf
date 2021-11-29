const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  entry: "./src/App.tsx",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "production",
  plugins: [
    new CopyWebpackPlugin({ patterns: ["src/index.html", "src/favicon.svg"] })
  ],
  resolve: {
    extensions: [".js", ".ts", ".tsx"],
    modules: ["src", "node_modules"],
  },
  target: ["web", "es2020"],
  module: {
    rules: [
      {
        test: /\.ts(x?)$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "ts-loader"
          }
        ]
      }
    ]
  },
  experiments: {
    asyncWebAssembly: true,
    topLevelAwait: true,
  },
};
