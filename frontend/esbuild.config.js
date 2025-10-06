    const esbuild = require('esbuild');

    esbuild.build({
        entryPoints: ['src/xrf-map-plot.ts',
            'src/proposals-index.ts',
            'src/samples-index.ts',
            'src/index.ts'
        ],
        bundle: true,
        outdir: 'static/',
        platform: 'browser',
        format: 'esm', // or 'iife' if you prefer a self-executing function
        minify: true, // Optional: for production builds
        sourcemap: true, // Optional: for debugging
    }).then(() => {
        console.log('esbuild finished bundling ECharts example!');
    }).catch(() => {
        process.exit(1);
    });
