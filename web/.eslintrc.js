const path = require('path');

module.exports = {
  root: true,

  extends: [
    'eslint:recommended',
    'plugin:react/recommended',
    'plugin:react/jsx-runtime',
    'plugin:tailwindcss/recommended',
    'plugin:prettier/recommended',
  ],

  ignorePatterns: ['node_modules/', 'dist/', '.direnv/'],
  settings: {
    tailwindcss: {
      config: path.resolve(__dirname, 'tailwind.config.js'), // buggy bullshit
    },
    react: {
      version: 'detect',
    },
  },

  env: {
    browser: true,
    node: true,
  },

  overrides: [
    {
      files: ['*.ts', '*.tsx', '*.js', '*.jsx'],
      parser: '@typescript-eslint/parser',
    },
  ],
};
