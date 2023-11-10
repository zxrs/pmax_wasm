const webpack = require("webpack");

module.exports = {
    entry: ["./index.mjs"],
    plugins: [
        new webpack.ProvidePlugin({
            Buffer: ["buffer", "Buffer"]
        }),
        new webpack.ProvidePlugin({process: "process/browser"}),
    ],
    resolve: {
        fallback: {
            "buffer": require.resolve("buffer")
        }
    },
    externals: {
        "wasmer_wasi_js_bg.wasm": true
    },
    output: {
        library: "generate_jpg",
        libraryTarget: "window",
        libraryExport: "default",
    }
};
