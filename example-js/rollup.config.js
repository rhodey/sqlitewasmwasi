import { nodeResolve } from '@rollup/plugin-node-resolve'

export default {
  input: 'index.js',
  external: (id) => id.startsWith('wasi:'),
  output: {
    file: 'dist/bundle.js',
    format: 'esm',
  },
  plugins: [nodeResolve()],
}
