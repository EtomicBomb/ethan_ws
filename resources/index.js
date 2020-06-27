var canvas = document.getElementById("canvas").getContext("2d");
canvas.font = '50px serif';

var width = 500;
var height = 500;

var blockWidth = 50;
var blockHeight = 50;

var x = (width-blockWidth)*Math.random();
var y = (height-blockHeight)*Math.random();

var vx = 3;
var vy = 3.1;

setInterval(function() {
    x += vx;
    y += vy;

    if (x+blockWidth > width || x < 0) {
        vx *= -1;
    }
    if (y+blockHeight > height || y < 0) {
        vy *= -1;
    }

    canvas.clearRect(0,0,width,height);
    canvas.fillRect(x,y, blockWidth, blockHeight);
}, 20);