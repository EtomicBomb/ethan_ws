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
socket.onopen = function() {
    socket.send("null"); // how we request a new game state
};

socket.onmessage = function(msg) {
    gameState = JSON.parse(msg.data);
    display();
};

var gameState;

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

canvas.onclick = function(event) {
    var boxX = Math.floor(event.pageX/size);
    var boxY = Math.floor(event.pageY/size);
    colorClicked(gameState.field[boxY][boxX]);
};

function colorClicked(color) {
    console.log("clicked on "+color);

    var toSend = JSON.stringify({
                         state: gameState,
                         move: color,
                     });
    console.log(toSend);
    socket.send(toSend);
}

function display() {
    canvas.width = width*size;
    canvas.height = height*size;

    for (var y=0; y<height; y++) {
        for (var x=0; x<width; x++) {

            context.fillStyle = gameState.field[y][x];
            context.fillRect(x*size, y*size, size, size);
        }
    }
}

//function Map(width, height) {
//    this.inner = [];
//    var x, y, row, color;
//
//    // top left corner case
//    row = [];
//    row[0] = randomColor();
//
//    // handle all the x's for y=0;
//    for (x=1; x<width; x++) {
//        color = randomColor();
//        while (row[x-1] == color) color = randomColor();
//        row[x] = color;
//    }
//    this.inner.push(row);
//
//    for (y=1; y<height; y++) {
//        // handle the x=0 case
//        row = [];
//        color = randomColor();
//        while (this.inner[y-1][0] == color) color = randomColor();
//        row[0] = color;
//
//        for (x=1; x<width; x++) {
//            color = randomColor();
//            while (row[x-1] == color || this.inner[y-1][x] == color) color = randomColor();
//            row[x] = color;
//        }
//        this.inner.push(row);
//    }
//
//
//    this.display = function() {
//        // puts the stuff on canvas
//        canvas.width = width*size;
//        canvas.height = height*size;
//
//        for (var y=0; y<height; y++) {
//            for (var x=0; x<width; x++) {
//
//                context.fillStyle = this.inner[y][x];
//                context.fillRect(x*size, y*size, size, size);
//            }
//        }
//    };
//}
//
//function randomColor() {
//    var index = Math.random()*colors.length;
//    return colors[index|0];
//}