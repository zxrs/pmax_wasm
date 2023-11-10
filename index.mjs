import {init, WASI} from "@wasmer/wasi";

let module;

(async () => {
    module = await WebAssembly.compileStreaming(fetch("pmax_wasm.wasm"));
})();

const generate_jpg = async (jpg, quality, size, delete_exif, denoise) => {
    await init();

    const wasi = new WASI({
        env: {},
        args: ["", `${quality}`, `${size}`, `${delete_exif}`, `${denoise}`],
    });
    wasi.instantiate(module, {});

    let input = wasi.fs.open("/input", {read: true, write: true, create: true});
    input.write(jpg);
    input.seek(0);

    const exitcode = wasi.start();
    const stderr = wasi.getStderrString();
    console.log(`${stderr} (exit code: ${exitcode})`);

    if (exitcode != 0) {
        return;
    }

    wasi.fs.removeFile("/input");

    const output = wasi.fs.open("/output", {read: true, write: false, create: false});
    const compressed_jpg = output.read();

    wasi.fs.removeFile("/output");

    wasi.free();

    return compressed_jpg;
};

export default generate_jpg
