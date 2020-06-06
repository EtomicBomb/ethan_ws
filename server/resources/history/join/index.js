let socket = new WebSocket("ws://ethan.ws/history");
socket.onclose = () => console.log("socket closed");
socket.onmessage = msg => {
    let data = JSON.parse(msg.data);
    console.log("received " + data.kind);
};

function joinGameButtonHandler() {
    let id = document.getElementById("joinGameIdInput").value;
    let username = document.getElementById("username").value;

    socket.send(JSON.stringify({
        kind: "join",
        username: username,
        id: id,
    }));
}