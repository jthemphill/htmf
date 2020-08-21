const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "production",
  plugins: [
    new CopyWebpackPlugin({ patterns: ["index.html"] })
  ],
  resolve: {
    extensions: [".js", ".ts", ".tsx"],
    modules: ["src", "node_modules"],
  },
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
      },
      {
        test: /bot\.worker\.ts$/,
        use: [
          { loader: "worker-loader" },
          { loader: "ts-loader" },
        ]
      },
    ]
  },
  externals: {
    "react": "React",
    "react-dom": "ReactDOM",
  },
};
