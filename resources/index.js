var canvas = document.getElementById("canvas").getContext("2d");
canvas.font = '50px serif';

var width = 500;
var height = 500;

var blockWidth = 20;
var blockHeight = 20;

var x = 100;
var y = 100;

var vx = 3;
var vy = 5;

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