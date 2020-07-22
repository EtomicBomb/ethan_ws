const DISPLAY_MODE = "grid";
const PLACES = ["downCards", "leftCards", "upCards", "rightCards"];

let selected = {};

let socket = new WebSocket("ws://ethan.ws/pusoy");
socket.onclose = () => console.log("socket closed");

socket.onopen = () => {

};

socket.onmessage = (event) => {
    let data = JSON.parse(event.data);

    console.log("received " + data.kind);
    console.log(data);

    if (data.kind == "createSuccess") {
        document.getElementById("landingScreen").style.display = "none";
        document.getElementById("lobbyScreen").style.display = DISPLAY_MODE;
    
        document.getElementById("hostLabel").innerText = data.host + "'s pusoy game";
        document.getElementById("gameIdLabel").innerText = "game id: " + data.gameId;

    } else if (data.kind == "joinSuccess") {
        document.getElementById("joinGameScreen").style.display = "none";
        document.getElementById("lobbyScreen").style.display = DISPLAY_MODE;
        document.getElementById("begin").style.display = "none";

        document.getElementById("hostLabel").innerText = data.host + "'s pusoy game";
        document.getElementById("gameIdLabel").innerText = "game id: " + data.gameId;

    } else if (data.kind == "refreshLobby") {
        let text = data.players.join("\n");

        document.getElementById("playerList").value = text;

    } else if (data.kind == "invalidGameId") {
        document.getElementById("gameIdInput").style.borderColor = "red";
    
    } else if (data.kind == "begin") {
        document.getElementById("lobbyScreen").style.display = "none";
        document.getElementById("gameScreen").style.display = DISPLAY_MODE;

    } else if (data.kind == "transition") {
        let onTable = data.onTable;

        setCardCounts(data.cardCounts, data.yourId, data.hand, onTable);

        document.getElementById("submitButton").style.outline = "none";

        for (let place of PLACES) document.getElementById(place).style.outline = "none";
        document.getElementById(elementNameFromIndex(data.turnIndex, data.yourId)).style.outline = "4px solid gold";

        document.getElementById("submitButton").disabled = true;

    } else if (data.kind == "turnBrief") {
        document.getElementById("submitButton").disabled = false;

        let canPass = data.canPass;
        let possiblePlays = data.possiblePlays;

        let string = "";

        for (let play of possiblePlays) {
            string += play.kind + " - " + play.cards.join(" ") +"\n";
        }

    } else if (data.kind == "over") {

        document.getElementById("gameOverLabel").hidden = false;

    } else if (data.kind == "invalidPlay") {
        document.getElementById("submitButton").disabled = false;
        document.getElementById("submitButton").style.outline = "2px solid red";

    } else if (data.kind == "hostAbandoned") {

    } else {
        console.log("unknown message kind "+data.kind);
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
        document.getElementById("landingScreen").style.display = "none";
        document.getElementById("joinGameScreen").style.display = DISPLAY_MODE;
    
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


document.getElementById("submitButton").onclick = () => {
    document.getElementById("submitButton").disabled = true; // double click bad

    let cards = [];
    for (let cardText in selected) {
        if (selected[cardText]) cards.push(cardText);
    }

    socket.send(JSON.stringify({
        kind: "playCardsArray",
        cards: cards,
    }));
};

function setCardCounts(counts, yourId, hand, onTable) {
    selected = {};

    document.getElementById("downCards").innerHTML = "";
    for (let cardText of hand) {
        let cardImage = document.createElement("img");
        cardImage.src = "http://ethan.ws/pusoy/cardFront.png";
        cardImage.style.height = "3.5em";
        cardImage.style.width = "2.25em";

        let labelDiv = document.createElement("div");
        labelDiv.appendChild(document.createTextNode(cardText));
        labelDiv.style.position = "absolute";
        labelDiv.style.top = "50%";
        labelDiv.style.left = "50%";
        labelDiv.style.transform = "translate(-50%, -50%)";

        let container = document.createElement("div");
        container.style.textAlign = "center";
        container.style.position = "relative";
        container.style.height = "3.5em"
        container.style.width = "2.25em";
        //container.style.pointerEvents = "none";
        container.style.userSelect = "none";

        container.appendChild(cardImage);
        container.appendChild(labelDiv);

        let isSelected = false;
        container.onclick = () => {
            isSelected = !isSelected;
            document.getElementById("submitButton").style.outline = "none";

            selected[cardText] = isSelected;
            container.style.transform = isSelected? "translateY(-25%)" : "none";
        };

        document.getElementById("downCards").appendChild(container);
    }

    document.getElementById("onTable").innerHTML = "";
    for (let cardText of onTable) {
        let cardImage = document.createElement("img");
        cardImage.src = "http://ethan.ws/pusoy/cardFront.png";
        cardImage.style.height = "3.5em"
        cardImage.style.width = "2.25em";

        let labelDiv = document.createElement("div");
        labelDiv.appendChild(document.createTextNode(cardText));
        labelDiv.style.position = "absolute";
        labelDiv.style.top = "50%";
        labelDiv.style.left = "50%";
        labelDiv.style.transform = "translate(-50%, -50%)";

        let container = document.createElement("div");
        container.style.textAlign = "center";
        container.style.position = "relative";
        container.style.height = "3.5em"
        container.style.width = "2.25em";
        container.style.pointerEvents = "none";
        container.style.userSelect = "none";
        
        container.appendChild(cardImage);
        container.appendChild(labelDiv);

        document.getElementById("onTable").appendChild(container);
    }

    document.getElementById("leftCards").innerHTML = "";
    for (let i=0; i<counts[(yourId+1)%4]; i++) {
        let cardBack = document.createElement("img");
        cardBack.src = "http://ethan.ws/pusoy/cardBack.png";
        cardBack.style.height = "3.5em"
        cardBack.style.width = "2.25em";
    
        document.getElementById("leftCards").appendChild(cardBack);
    }

    document.getElementById("upCards").innerHTML = "";
    for (let i=0; i<counts[(yourId+2)%4]; i++) {
        let cardBack = document.createElement("img");
        cardBack.src = "http://ethan.ws/pusoy/cardBack.png";
        cardBack.style.height = "3.5em"
        cardBack.style.width = "2.25em";
    
        document.getElementById("upCards").appendChild(cardBack);
    }

    document.getElementById("rightCards").innerHTML = "";
    for (let i=0; i<counts[(yourId+3)%4]; i++) {
        let cardBack = document.createElement("img");
        cardBack.src = "http://ethan.ws/pusoy/cardBack.png";
        cardBack.style.height = "3.5em"
        cardBack.style.width = "2.25em";
    
        document.getElementById("rightCards").appendChild(cardBack);
    }

}

function elementNameFromIndex(index, yourId) {

    let i = (index - yourId) % 4;
    if (i < 0) i += 4;

    return PLACES[i];
}