const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, 'dist');

module.exports = {
  mode: 'development',
  entry: {
    index: './index.js',
  },
  output: {
    path: dist,
    filename: '[name].js',
  },
  devServer: {
    static: {
      directory: dist,
    },
  },
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html',
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      outDir: 'pkg',
    }),
  ],
};
