var colors = [
    "red",
    "green",
    "blue",
    "yellow",
    "black",
    "purple",
];

var BLINK = {
    red: "fuchsia",
    green: "lime",
    blue: "lightblue",
    yellow: "orange",
    black: "grey",
    purple: "rebeccapurple",
};

const BLINK_PERIOD = 750;

var socket = new WebSocket("ws://ethan.ws/filler");

var gameState;

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

socket.onmessage = function(msg) {
    gameState = JSON.parse(msg.data);
    display();
};

canvas.onclick = function(event) {
    var boxX = Math.floor(event.offsetX/getSize());
    var boxY = Math.floor(event.offsetY/getSize());
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

let blink = true;

setInterval(() => {

    display(blink);
    blink = !blink;
}, BLINK_PERIOD);


function display(blink) {
    // display our available colors
    document.getElementById("score").innerText = gameState.leftTerritory.length + "(you) vs " + gameState.rightTerritory.length + "(ai)";

    for (let color of colors) {
        document.getElementById(color).style.display = gameState.availableColors.includes(color)? "inline-block" : "none";
    }

    let width = gameState.field[0].length;
    let height = gameState.field.length;
    // display field

    let size = getSize();
    canvas.width = width*size;
    canvas.height = height*size;

    for (var y=0; y<height; y++) {
        for (var x=0; x<width; x++) {

            let color = gameState.field[y][x];

            // if (blink && surroundingIncludes(x, y)) {
            //     color = BLINK[color];
            // } 

            context.strokeStyle = color;
            context.fillStyle = color;
            context.fillRect(x*size, y*size, size, size);
        }
    }
}

function surroundingIncludes(x, y) {
    for (let square of gameState.surrounding) {
        if (square.x == x && square.y == y) return true;
    }

    return false;
}

function getSize() {
    // let width = gameState.field[0].length;
    // let height = gameState.field.length;

    // return Math.round(Math.min(window.innerWidth, window.innerHeight) / Math.max(width, height));
    return 50;
}

function rgb(r, g, b) {
    return "rgb("+r+","+g+","+b+")";
}

function clone(a) {
    return JSON.parse(JSON.stringify(a));
}