/// @ts-check
/// <reference types="node" />

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

/** @type {import('webpack').Configuration[]} */
module.exports = [
  {
    entry: {
      app: path.resolve(__dirname, '../src/app/index.tsx'),
    },

    target: 'web',

    output: {
      path: path.resolve(__dirname, '../dist/web/'),
    },

    plugins: [
      new HtmlWebpackPlugin({
        template: path.resolve(__dirname, '../src/app/index.html'),
      }),
    ],
  },
];
