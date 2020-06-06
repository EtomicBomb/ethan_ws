let socket = new WebSocket("ws://ethan.ws/history");
socket.onclose = () => console.log("socket closed");
socket.onmessage = msg => {
    let data = JSON.parse(msg.data);
    console.log("received " + data.kind);
};

function createGameButtonHandler() {
    socket.send(JSON.stringify({
        kind: "create",
        username: document.getElementById("username").value,
        settings: document.querySelector("input[name='gameKind']:checked").id,
    }));
}