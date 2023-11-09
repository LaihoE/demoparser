
async function run_wasm() {
    // Load the Wasm file by awaiting the Promise returned by `wasm_bindgen`
    // `wasm_bindgen` was imported in `index.html`
    await wasm_bindgen('../pkg/demoparser2_bg.wasm');
    var myWorkers = [new Worker('./worker.js')];
    // Create a worker in JS. The worker also uses Rust functions


    
    const numWorkers = 24; // Adjust based on your requirements
    const workers = [];
    for (let i = 0; i < numWorkers; i++) {
        workers.push(new Worker('./worker.js'));
    }

    document.getElementById("file_picker").addEventListener(
        "change",
        function () {        

            const tbl = document.createElement("table");
            const tblBody = document.createElement("tbody");
            tbl.appendChild(tblBody);
            document.body.appendChild(tbl);
            tbl.setAttribute("border", "2");
            var column_names = ["name", "kills", "round", "file"];
            var table = document.createElement("TABLE");  //makes a table element for the page
            table.id = "table";  // Add this line to set the id
            var header = table.createTHead();
            var headerRow = header.insertRow(0);
            for (var i = 0; i < column_names.length; i++) {
                headerRow.insertCell(i).innerHTML = column_names[i];
            }
            document.body.append(table);

            let tasks = []
            for (let i = 0; i < this.files.length; i++){
                tasks.push(this.files[i])
            }
            document.getElementById("health").max = tasks.length;

            // Divide tasks among workers
            tasks.forEach((task, index) => {
                console.log(task);
                console.log(index);
            workers[index % numWorkers].postMessage({ file: task });
            });
            var weaponName = document.getElementById("wanted-gun").value;
            var nRoundKills = document.getElementById("round-kills").value;

            workers.forEach((worker, index) => {
                worker.onmessage = function (e) {
                  // Process the result from the worker
                  generateTableFromData(e.data, weaponName, nRoundKills, "asdf")
                };
              });
        },
        false
    );
}

function generateTableFromData(events, weaponName, nRoundKills, fileName, ) {
    document.getElementById("health").value += 1;

    let maxRound = Math.max(...events.map(events => events.get("total_rounds_played")))
    const wantedRows = [];

    for (let round = 0; round <= maxRound; round++){
        const killsPerPlayer = {};
        let killsThisRound = events.filter(kill => kill.get("total_rounds_played") == round)

        killsThisRound.forEach(item => {
            const attackerName = item.get("attacker_name");
            const kills = killsPerPlayer[attackerName] || 0;
            if (item.get("weapon") == weaponName){
                killsPerPlayer[attackerName] = kills + 1;
            }
        });
        for (let [key, value] of Object.entries(killsPerPlayer)) {
            if (value == nRoundKills){
                wantedRows.push({"name": key, "kills": value, "round": round, "file": fileName});
            }
        }
    }   
    let table = document.getElementById("table");

    for (var i = 0; i < wantedRows.length; i++) {
        var row = table.insertRow(i);
        row.insertCell(0).innerHTML = wantedRows[i].name;
        row.insertCell(1).innerHTML = wantedRows[i].kills;
        row.insertCell(2).innerHTML = wantedRows[i].round;
        row.insertCell(3).innerHTML = wantedRows[i].file;
    }
}
run_wasm();
