// We compiled with `--target no-modules`, which does not create a module. The generated bindings
// can be loaded in web workers in all modern browsers.
import init, { parse_events, initThreadPool, parse  } from './pkg/demoparser2.js';
await init();

async function run_in_worker() {
    // Load the Wasm file by awaiting the Promise returned by `wasm_bindgen`
    await wasm_bindgen('../pkg/demoparser2_bg.wasm');
    console.log("worker.js has loaded Wasm file → Functions defined with Rust are now available");
}

// run_in_worker();


onmessage = async function (e) {

    await initThreadPool(12);
    const file = e.data.file;
    const reader = new FileReader();
    
    console.log("SET",2, "THREADS")
    reader.onload = function (event) {
        const arrayBuffer = event.target.result;
        const uint8Array = new Uint8Array(arrayBuffer);
        // Perform your operations with uint8Array here
        // let result = parse(uint8Array);

        let result = parse_events(uint8Array, "player_death");
        postMessage(result);
    };

    // Read the file as an ArrayBuffer
    reader.readAsArrayBuffer(file);
};


