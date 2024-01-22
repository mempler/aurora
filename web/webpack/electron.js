/// @ts-check
/// <reference types="node" />

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

/** @type {import('webpack').Configuration[]} */
module.exports = [
  //
  // Primary electron process that has full access to the system.
  //
  {
    entry: {
      'electron-main': path.resolve(__dirname, '../src/electron/main/index.ts'),
    },

    target: 'electron-main',

    externalsPresets: {
      electronMain: true,
    },

    output: {
      path: path.resolve(__dirname, '../dist/electron/'),
    },
  },

  //
  // Preload script that has access to the electron API to bridge to the
  // renderer process.
  //
  {
    entry: {
      'electron-preload': path.resolve(__dirname, '../src/electron/preload/index.ts'),
    },

    target: 'electron-preload',

    externalsPresets: {
      electronPreload: true,
    },

    output: {
      path: path.resolve(__dirname, '../dist/electron/'),
    },
  },

  //
  // The renderer process is the web page itself. but there are apparently a few distinctions between
  // the web page and the renderer process. The renderer process has access to the DOM and the
  // electron API, but not the node API. The web page has access to the DOM but not the electron API.
  //
  {
    entry: {
      'electron-renderer': path.resolve(__dirname, '../src/electron/renderer/index.tsx'),
    },

    target: 'electron-renderer',

    externalsPresets: {
      electronRenderer: true,
    },

    output: {
      path: path.resolve(__dirname, '../dist/electron/'),
    },

    plugins: [
      new HtmlWebpackPlugin({
        template: path.resolve(__dirname, '../src/electron/index.html'),
      }),
    ],
  },
];
