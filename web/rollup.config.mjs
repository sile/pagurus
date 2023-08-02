import typescript from '@rollup/plugin-typescript';
import pkg from './package.json';

const banner = `/**
 * ${pkg.name}
 * ${pkg.description}
 * @version: ${pkg.version}
 * @author: ${pkg.author}
 * @license: ${pkg.license}
 **/
`;

export default [
  {
    input: 'src/pagurus.ts',
    plugins: [
      typescript({module: "esnext"})
    ],
    output: {
      sourcemap: false,
      file: './dist/pagurus.mjs',
      format: 'module',
      name: 'Pagurus',
      banner: banner,
    }
  },
  {
    input: 'src/pagurus.ts',
    plugins: [
      typescript({module: "esnext"})
    ],
    output: {
      sourcemap: false,
      file: './dist/pagurus.js',
      format: 'umd',
      name: 'Pagurus',
      banner: banner,
    }
  }
];
