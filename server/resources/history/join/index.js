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
        

    } else if (data.kind == "initialStuff") {
        document.getElementById("lobby").style.display = "none";
        document.getElementById("game").style.display = "block";

        let question = data.question;         
        displayQuestion(question.definition, question.terms);

    } else if (data.kind == "updateStuff") {
        let wasCorrect = data.wasCorrect;

        console.log("was correct " + wasCorrect)

        document.getElementById("score").innerText = "Score: " + data.score;

        let newQuestion = data.newQuestion;
        displayQuestion(newQuestion.definition, newQuestion.terms);
    }
};

function displayQuestion(definition, terms) {
    document.getElementById("definition").innerText = definition;

    for (let i=0; i<4; i++) {
        document.querySelector("label[for='answer"+i+"']").innerText = terms[i];
        document.getElementById("answer"+i).disabled = false;
    }
}

function submitAnswerHandler(answer) {
    // let answer = +document.querySelector("input[type=radio]:checked").id.slice(-1);
    socket.send(JSON.stringify({
        kind: "submitAnswer",
        answer: answer,
    }))

    for (let i=0; i<4; i++) {
        document.getElementById("answer"+i).disabled = true;
    }

}

function joinGameButtonHandler() {
    let username = document.getElementById("username").value;
    let id = +document.getElementById("joinGameIdInput").value;

    socket.send(JSON.stringify({
        kind: "join",
        username: username,
        id: id,
    }));
}