const { merge } = require('webpack-merge');
const path = require('path');
const common = require('./webpack.config.js');

module.exports = merge(common, {
  mode: "development",
  devtool: 'source-map',
  devServer: {
    static: {
      directory: path.join(__dirname, '../../docs/'), // eslint-disable-line
    },
    port: 9000,
    open: true,
    // host: '0.0.0.0',
    // disableHostCheck: true
  }
});
