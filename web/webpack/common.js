/// @ts-check
/// <reference types="node" />

const path = require('path');

const isDev = process.env.NODE_ENV === 'development';

/** @type {import('webpack').Configuration} */
module.exports = {
  mode: isDev ? 'development' : 'production',

  output: {
    filename: '[name].js',
  },

  resolve: {
    extensions: ['.ts', '.js', '.tsx', '.jsx'],
    alias: {
      '@': path.resolve(__dirname, '../src'),
    },
  },

  devServer: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        exclude: [/node_modules/],
        loader: 'builtin:swc-loader',
        options: {
          sourceMap: true,
          jsc: {
            parser: {
              syntax: 'typescript',
              jsx: true,
            },
            externalHelpers: true,
            preserveAllComments: false,
            transform: {
              react: {
                runtime: 'automatic',
                pragma: 'React.createElement',
                pragmaFrag: 'React.Fragment',
                throwIfNamespace: true,
                useBuiltins: false,
              },
            },
          },
        },
        type: 'javascript/auto',
      },

      {
        test: /\.css$/,
        exclude: [/node_modules/],
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: ['postcss-preset-env', 'tailwindcss'],
              },
            },
          },
        ],
        type: 'css',
      },
    ],
  },
};
