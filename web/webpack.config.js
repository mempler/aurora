/// @ts-check
/// <reference types="node" />

const { merge } = require('webpack-merge');

const wantWeb = process.env.WEBPACK_TARGET?.toLowerCase() === 'web';
const wantElectron = process.env.WEBPACK_TARGET?.toLowerCase() === 'electron';
const wantAll = process.env.WEBPACK_TARGET?.toLowerCase() === 'all';

const configs = [];

if (wantWeb || wantAll) {
  configs.push(...require('./webpack/web'));
}

if (wantElectron || wantAll) {
  configs.push(...require('./webpack/electron'));
}

console.log(`Webpack target: ${process.env.WEBPACK_TARGET}`);
console.log(`Webpack mode: ${process.env.NODE_ENV}`);

const commonConfig = require('./webpack/common');

/** @type {import('webpack').Configuration[]} */
module.exports = configs.map((config) => merge(commonConfig, config));

//const util = require('util');
//console.log(util.inspect(module.exports, true, 10, true));
