var socket = new WebSocket("ws://ethan.ws");

socket.onopen = function() {
    console.log("opened");
};

socket.onmessage = function(msg) {
    document.getElementById("serverResponse").value = msg.data;
    console.log(msg);
};

socket.onclose = function(r) {
    console.log("closed");
    document.getElementById("serverResponse").value = "Error: connection closed " + r.code;
}

function clickHandler() {
    var response = document.getElementById("keywordInput").value
        + "|" + document.getElementById("yearRangeMin").value
        + "|" + document.getElementById("yearRangeMax").value
        + "|" + document.getElementById("socialChecked").checked
        + "|" + document.getElementById("politicalChecked").checked
        + "|" + document.getElementById("economicChecked").checked;

    console.log(response);

    socket.send(response);
}

//document.getElementById("input").addEventListener("keyup", function(event) {
//    if (event.key == "Enter") {
//        clickHandler();
//    }
//});

