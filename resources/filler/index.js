var colors = [
    "red",
    "green",
    "blue",
    "yellow",
    "black",
    "purple",
];

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

var colorButtonPalette = document.getElementById("palette")
for (var i=0; i<colors.length; i++) {
    var button = document.createElement("button");
    button.style["background-color"] = colors[i];
    button.onclick = function(event) { buttonOnClick(colors[i], event); };
    colorButtonPalette.appendChild(button);
}

function buttonOnClick(color, event) {
    console.log(color.style["background-color"] + "x: "+ event.pageX + " y:" + event.pageY);
}

var size = 50;

var map = new Map(8, 7);
map.display();

canvas.onclick = function(event) {
    var boxX = Math.floor(event.pageX/size);
    var boxY = Math.floor(event.pageY/size);
    console.log("x: "+boxX+" y: "+boxY);
};

function Map(width, height) {
    this.inner = [];
    var x, y, row, color;

    // top left corner case
    row = [];
    row[0] = randomColor();

    // handle all the x's for y=0;
    for (x=1; x<width; x++) {
        color = randomColor();
        while (row[x-1] == color) color = randomColor();
        row[x] = color;
    }
    this.inner.push(row);

    for (y=1; y<height; y++) {
        // handle the x=0 case
        row = [];
        color = randomColor();
        while (this.inner[y-1][0] == color) color = randomColor();
        row[0] = color;

        for (x=1; x<width; x++) {
            color = randomColor();
            while (row[x-1] == color || this.inner[y-1][x] == color) color = randomColor();
            row[x] = color;
        }
        this.inner.push(row);
    }


    this.display = function() {
        // puts the stuff on canvas
        canvas.width = width*size;
        canvas.height = height*size;

        for (var y=0; y<height; y++) {
            for (var x=0; x<width; x++) {

                context.fillStyle = this.inner[y][x];
                context.fillRect(x*size, y*size, size, size);
            }
        }
    };
}

function randomColor() {
    var index = Math.random()*colors.length;
    return colors[index|0];
}