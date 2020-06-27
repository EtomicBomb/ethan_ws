var colors = [
    "red",
    "green",
    "blue",
    "yellow",
    "black",
    "purple",
];

var width = 8;
var height = 7;
var size = 50;

var socket = new WebSocket("ws://ethan.ws/filler");

var gameState;

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

socket.onmessage = function(msg) {
    gameState = JSON.parse(msg.data);
    display();
};

canvas.onclick = function(event) {
    var boxX = Math.floor(event.offsetX/size);
    var boxY = Math.floor(event.offsetY/size);
    console.log(event.pageX + " " + event.pageY);

    colorClicked(gameState.field[boxY][boxX]);
};

function colorClicked(color) {
    if (gameState.availableColors.includes(color)) {
        console.log("clicked on "+color);
        socket.send(color);
    } else {
        console.log("clicked invalid color "+color);
    }
}

function display() {
    // display our available colors

    document.getElementById("score").innerText = gameState.leftTerritory.length + "(you) vs " + gameState.rightTerritory.length + "(ai)";

    for (let color of colors) {
        document.getElementById(color).style.display = gameState.availableColors.includes(color)? "inline-block" : "none";
    }

    // display field
    canvas.width = width*size;
    canvas.height = height*size;

    for (var y=0; y<height; y++) {
        for (var x=0; x<width; x++) {

            context.fillStyle = gameState.field[y][x];
            context.fillRect(x*size, y*size, size, size);
        }
    }
}