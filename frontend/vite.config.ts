import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      $lib: resolve('./src/lib'),
      $data: resolve('./src/data'),      
    }
  },
  plugins: [svelte()]
})
