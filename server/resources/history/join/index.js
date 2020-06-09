let socket = new WebSocket("ws://ethan.ws/history");
socket.onclose = () => console.log("socket closed");
socket.onmessage = msg => {
    let data = JSON.parse(msg.data);
    console.log("received " + data.kind);

    if (data.kind == "invalidGameId") {
        document.getElementById("errorLabel").innerText = "That lobby wasn't found";

    } else if (data.kind == "joinSuccess") {
        document.getElementById("joinMenu").style.display = "none";
        document.getElementById("lobby").style.display = "block";
        document.getElementById("hostName").innerText = data.hostName + "'s Lobby";
        
    } else if (data.kind == "refreshLobby") {
        document.getElementById("members").value = data.users.join("\n");

    } else if (data.kind == "hostAbandoned") {
        

    }
};

function joinGameButtonHandler() {
    let username = document.getElementById("username").value;
    let id = +document.getElementById("joinGameIdInput").value;

    socket.send(JSON.stringify({
        kind: "join",
        username: username,
        id: id,
    }));
}