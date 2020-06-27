let socket = new WebSocket("ws://ethan.ws/pusoy");
socket.onclose = () => console.log("socket closed");

socket.onopen = () => {


};

socket.onmessage = (event) => {
    let data = JSON.parse(event.data);

    console.log("received " + data.kind);

    if (data.kind == "createSuccess") {
        document.getElementById("landing").style.display = "none";
        document.getElementById("lobby").style.display = "block";
    
        document.getElementById("hostLabel").innerText = data.host + "'s pusoy game";
        document.getElementById("gameIdLabel").innerText = "game id: " + data.gameId;

    } else if (data.kind == "joinSuccess") {
        document.getElementById("joinGame").style.display = "none";
        document.getElementById("lobby").style.display = "block";
        document.getElementById("begin").style.display = "none";

        document.getElementById("hostLabel").innerText = data.host + "'s pusoy game";
        document.getElementById("gameIdLabel").innerText = "game id: " + data.gameId;

    } else if (data.kind == "refreshLobby") {

        let text = data.players.join("\n");

        document.getElementById("playerList").value = text;

    } else if (data.kind == "invalidGameId") {
        document.getElementById("gameIdInput").style.borderColor = "red";
    
    } else if (data.kind == "beginGame") {

    } else if (data.kind == "hostAbandoned") {
    
    }
};

document.getElementById("begin").onclick = () => {
    socket.send(JSON.stringify({
        kind: "begin",
    }));    
};

document.getElementById("startGameButton").onclick = () => {
    let username = document.getElementById("username").value;

    if (username) {
        socket.send(JSON.stringify({
            kind: "create",
            username: username,
        }));
    } else {
        document.getElementById("username").style.borderColor = "red";
    }
};

document.getElementById("joinGameButton").onclick = () => {
    let username = document.getElementById("username").value;

    if (username) {
        document.getElementById("landing").style.display = "none";
        document.getElementById("joinGame").style.display = "block";
    
    } else {
        document.getElementById("username").style.borderColor = "red";
    }
};


document.getElementById("submitGameId").onclick = () => {
    let gameId = document.getElementById("gameIdInput").value;
    let username = document.getElementById("username").value;

    socket.send(JSON.stringify({
        kind: "join",
        username: username,
        gameId: gameId,
    }));
};
