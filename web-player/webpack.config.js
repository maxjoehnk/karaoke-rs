const CopyWebpackPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
var ImageminPlugin = require('imagemin-webpack-plugin').default;
const path = require('path');

module.exports = {
  entry: "./static/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "./../embed/static"),
    filename: "bootstrap.js",
    publicPath: "/static/",
  },
  mode: "development",
  plugins: [
    process.env.NODE_ENV === 'production' ? new CleanWebpackPlugin() : false,
    new CopyWebpackPlugin([
      'static/background.png',
      'static/favicon.ico',
      { from: 'index.html', to: "./../templates/player.html" }
    ]),
    new ImageminPlugin({
      test: /\.(jpe?g|png|gif|svg)$/i,
      pngquant: {
        quality: '95-100'
      }
    })
  ].filter(Boolean),
  devServer: {
    host: '0.0.0.0',
    proxy: {
      '/player': 'http://127.0.0.1:8081',
    },
  },
};
