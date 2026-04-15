const path = require('path');

module.exports = {
  target: 'electron-renderer',
  devtool: 'source-map',
  entry: './src/renderer/index.tsx',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: {
          loader: 'ts-loader',
          options: { configFile: 'tsconfig.json' },
        },
        exclude: /node_modules/,
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    alias: { '@': path.resolve(__dirname, 'src') },
  },
  output: {
    filename: 'renderer.js',
    path: path.resolve(__dirname, 'dist'),
  },
};
