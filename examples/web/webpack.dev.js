const { merge } = require('webpack-merge');
const path = require('path');
const common = require('./webpack.config.js');

module.exports = merge(common, {
  mode: "development",
  devtool: 'source-map',
  devServer: {
    inline: true,
    contentBase: [
      path.join(__dirname, '../../docs/'), // eslint-disable-line
    ],
    watchContentBase: false,
    port: 9000,
    open: true,
    // host: '0.0.0.0',
    // disableHostCheck: true
  }
});
