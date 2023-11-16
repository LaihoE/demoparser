import init, { parse_events, initThreadPool  } from './pkg/demoparser2.js';

// We compiled with `--target no-modules`, which does not create a module. The generated bindings
// can be loaded in web workers in all modern browsers.

async function run_in_worker() {
    // Load the Wasm file by awaiting the Promise returned by `wasm_bindgen`
    await wasm_bindgen('../pkg/demoparser2_bg.wasm');
    console.log("worker.js has loaded Wasm file → Functions defined with Rust are now available");
}

run_in_worker();


onmessage = async function (e) {
    let result = parse_events(
        e.data.file,
        "rank_update",
    );
    postMessage(result);
};

