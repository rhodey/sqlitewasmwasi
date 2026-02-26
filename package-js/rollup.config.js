import { nodeResolve } from '@rollup/plugin-node-resolve'
import replace from '@rollup/plugin-replace'

export default {
  input: 'test.js',
  external: (id) => id.startsWith('wasi:'),
  output: {
    file: 'dist/bundle.js',
    format: 'esm',
  },
  plugins: [
    nodeResolve({
      browser: true,
      exportConditions: ['browser']
    }),
    replace({
      preventAssignment: true,
      'process.env.NODE_ENV': JSON.stringify('dev'),
    }),
  ],
}
