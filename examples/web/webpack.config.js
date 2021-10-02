const CopyPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  mode: 'production',
  entry: "./src/js/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "./dist"), // eslint-disable-line
    filename: "bootstrap.js",
    publicPath: "", // Automatic publicPath is not supported in this browser
  },
  plugins: [
    new CopyPlugin({ patterns: ['./src/www/index.html', './src/www/style.css'] })
  ],
  module: {
    rules: [
      {
        test: /.js$/,
        exclude: /node_modules/,
        use: [{
          loader: 'babel-loader',
          options: {
            presets: ['@babel/preset-env'],
            plugins: ['@babel/plugin-transform-runtime'],
          }
        }]
      }
    ],
    parser: {
      javascript: {
        // https://github.com/webpack/webpack/issues/11543#issuecomment-826938252
        worker: ["AudioWorklet from audio-worklet", "..."]
      }
    },
  },
  experiments: {
    // for use alias wasi_snapshot_preview1
    syncWebAssembly: true,
  },
  resolve: {
    extensions: ['.js', '.wasm'],
    modules: [
      "node_modules"
    ],
    alias: {
      // Import "fd_seek" from "wasi_snapshot_preview1" with Non-JS-compatible Func Signature (i64 as parameter)
      //  can only be used for direct wasm to wasm dependencies
      // webpack/lib/wasm-sync/WebAssemblyParser.js
      //  const JS_COMPAT_TYPES = new Set(["i32", "f32", "f64"]);
      // build for workaround patch examples/web/node_modules/webpack/lib/wasm-sync/WebAssemblyParser.js
      //  const JS_COMPAT_TYPES = new Set(["i32", "i64", "f32", "f64"]);
      "wasi_snapshot_preview1": path.resolve(__dirname, './src/js/wasi_snapshot_preview1.js'), // eslint-disable-line
      // https://github.com/webpack/webpack/issues/11543#issuecomment-826938252
      "audio-worklet": path.resolve(__dirname, "./src/js/audio-worklet.js"), // eslint-disable-line
    }
  },
  performance: {
    hints: false
  }
};
