
async function run_wasm() {
    // Load the Wasm file by awaiting the Promise returned by `wasm_bindgen`
    // `wasm_bindgen` was imported in `index.html`
    await wasm_bindgen('../pkg/demoparser2_bg.wasm');

    // Create a worker in JS. The worker also uses Rust functions
    var myWorker = new Worker('./worker2.js');

    document.getElementById("file_picker").addEventListener(
        "change",
        function () {

            let file = this.files[0];
            var startTime = performance.now()

            myWorker.postMessage({ file: file });
            myWorker.onmessage = function (e) {
                var endTime = performance.now()
                generateTableFromData(e.data)
            };
        },
        false
    );
}

function generateTableFromData(eventsNoFilter) {
    console.log(eventsNoFilter);
    let events = eventsNoFilter.filter(x => x.get("attacker_team_name") === x.get("user_team_name"));

    console.log(events);
    
    console.log("RAM RANCH")


    const tbl = document.createElement("table");
    const tblBody = document.createElement("tbody");

    tbl.appendChild(tblBody);
    document.body.appendChild(tbl);
    tbl.setAttribute("border", "2");

    var table = document.createElement("TABLE");  //makes a table element for the page
    var column_names = Array.from(events[0].keys());
    
    for (var i = 0; i < events.length; i++) {
        var row = table.insertRow(i);
        for (const [index, element] of column_names.entries()) {
            row.insertCell(index).innerHTML = events[i].get(element);
        }
    }

    var header = table.createTHead();
    var headerRow = header.insertRow(0);
    for (var i = 0; i < column_names.length; i++) {
        headerRow.insertCell(i).innerHTML = column_names[i];
    }
    document.body.append(table);
}

run_wasm();
