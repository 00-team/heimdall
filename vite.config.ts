import { defineConfig } from 'vite'
import type { WatcherOptions } from 'rollup'
import solidPlugin from 'vite-plugin-solid'

import tsconfigPaths from 'vite-tsconfig-paths'

let target = 'http://0.0.0.0:7000'
// let target = 'https://heimdall.00-team.org'

export default defineConfig(env => {
    let watch: WatcherOptions | null = null
    if (env.mode == 'development') {
        watch = {
            clearScreen: true,
        }
    }

    return {
        plugins: [tsconfigPaths(), solidPlugin({ hot: false })],
        root: 'app',
        server: {
            https: false,
            host: '0.0.0.0',
            port: 8000,
            proxy: {
                '/api/': {
                    target,
                    changeOrigin: true,
                },
                '/record/': {
                    target,
                    changeOrigin: true,
                },
                '/static/': {
                    target,
                    changeOrigin: true,
                },
            },
        },
        build: {
            target: 'esnext',
            outDir: '../web/dist',
            watch,
            assetsInlineLimit: 0,
            copyPublicDir: false,
            emptyOutDir: true,
        },
    }
})
