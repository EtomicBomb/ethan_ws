"use strict";

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

var width = window.innerWidth;
var height = window.innerHeight;

var keysDown = {};

canvas.width = width;
canvas.height = height;

window.addEventListener("resize", () => {
    width = window.innerWidth;
    height = window.innerHeight;

    canvas.width = width;
    canvas.height = height;
});


document.addEventListener("pointerlockchange", event => {
    if (document.pointerLockElement === canvas) {
        document.getElementById("lockWarning").style.display = "none";
    } else {
        document.getElementById("lockWarning").style.display = "block";
    }
});


/////////////////////////// ACTUAL GAME CODE //////////////////////////


var players = [];
var socket = new WebSocket("ws://ethan.ws/arena");

socket.onmessage = (msg) => {
    var data = JSON.parse(msg.data);

    players = [];
    for (var player of data) {
        players.push(new Circle(player.x, player.y, PLAYER_RADIUS, player.color));
    }
};

socket.onopen = () => {
    setInterval(() => {
        socket.send(JSON.stringify({
            x: playerX,
            y: playerY,
        }));
    }, 50);
};


var RECT_WIDTH = 5;

var TAU = 2*Math.PI;
//var FOV = TAU/2;
var FOV = TAU/10;
var ROTATE_ANGLE = TAU/20;
var PLAYER_RADIUS = 0.1;

var blue = {r:0,g:0,b:255};
var green = {r:0,g:255,b:0};
var boauns = {r:255,g:255,b:0};
var blang = {r:0,g:255,b:255};
var boam = {r:255,g:0,b:255};

var samInducedBruh = [
    new HorizontalLine(0, -5, Infinity, boam),

    new HorizontalLine(0, 0, 9, blue),
    new HorizontalLine(9, 0, 9, blue),
    new VerticalLine(0, 0, 9, blue),
    new VerticalLine(9, 0, 9, blue),
    new Circle(5, 5, 0.5, green),
    new Circle(7, 5, 0.5, boauns),
    new Circle(3, 1, 0.75, blang),
    new Circle(1, 6, 0.75, boam),
    new InsideCircle(5, 5, 0.5, blue),
    new InsideCircle(-30, 0, 10, green),
    new Circle(-30, 0, 10, green),
];


var theta = -0.753982236861552;
var playerX = 3;
var playerY = 3;

document.addEventListener("keydown", event => {
   if (event.key == "Enter") {
        canvas.requestPointerLock();
   }

    keysDown[event.key] = true;
});

document.addEventListener("keyup", event => keysDown[event.key] = false);

document.addEventListener("mousemove", event => {
    theta += 0.01 * ROTATE_ANGLE * event.movementX;
});

var touchMove = "none";
document.addEventListener("touchstart", event => {
    var x = event.touches[0].clientX;
    var y = event.touches[0].clientY;

    if (y < height/4) {
        touchMove = "forward";
    } else if (y > 3*height/4) {
        touchMove = "backward"
    } else {
        if (x < width/2) {
            touchMove = "left";
        } else {
           touchMove = "right";
        }
    }
});
document.addEventListener("touchend", event => {
    touchMove = "none";
});



window.requestAnimationFrame(drawFrame);

var lastElapsed;
function drawFrame(elapsed) {
    var step = 0.003*(elapsed - lastElapsed);
    lastElapsed = elapsed;

    if (keysDown["w"] || keysDown[","] || touchMove == "forward") {
        playerX += step*Math.cos(theta);
        playerY -= step*Math.sin(theta);
    }
    if (keysDown["s"] || keysDown["o"] || touchMove == "backward") {
        playerX -= step*Math.cos(theta);
        playerY += step*Math.sin(theta);
    }
    if (keysDown["a"]) {
        playerX += step*Math.sin(theta);
        playerY += step*Math.cos(theta);
    }
    if (keysDown["d"] || keysDown["e"]) {
        playerX -= step*Math.sin(theta);
        playerY -= step*Math.cos(theta);
    }
    if (keysDown["ArrowLeft"] || touchMove == "left") {
        theta -= step*ROTATE_ANGLE;
    }
    if (keysDown["ArrowRight"] || touchMove == "right") {
        theta += step*ROTATE_ANGLE;
    }


    context.fillStyle = "white";
    context.strokeStyle = "white";
    context.fillRect(0, 0, width, height);

    var cos, sin, distance, d, f, wallStuff, i;

    for (var screenX=0, checkAngle=theta-FOV; screenX<width; screenX += RECT_WIDTH, checkAngle += RECT_WIDTH*FOV/(width/2)) {
        cos = Math.cos(checkAngle);
        sin = Math.sin(checkAngle);
        distance = Infinity;

        var color = {r:255,g:255,b:255};

        for (var bruh of samInducedBruh.concat(players)) {
            d = bruh.intersect(playerX, playerY, cos, sin);
            if (d>0 && d<distance) {
                distance = d;
                color = bruh.color;
            }
        }

        wallStuff = height/2 - height/distance;

        f = map(distance, 0, 20, 1, 0);
        var colorString = rgb(color.r*f, color.g*f, color.b*f);


        context.fillStyle = colorString;
        context.strokeStyle = colorString;
        context.fillRect(screenX, wallStuff, RECT_WIDTH, height-2*wallStuff);
    }

    window.requestAnimationFrame(drawFrame);
}

function InsideCircle(h, k, r, color) {
    this.intersect = function(x0, y0, cos, sin) {
        var b = k*sin - h*cos + x0*cos - y0*sin;
        var descriminant = b*b - x0*x0 - y0*y0 + 2*h*x0 + 2*k*y0 - k*k - h*h + r*r;
        return descriminant < 0? -1 : -b + Math.sqrt(descriminant);
    };

    this.color = color;
}

function Circle(h, k, r, color) {
    this.intersect = function(x0, y0, cos, sin) {
        var b = k*sin - h*cos + x0*cos - y0*sin;
        var descriminant = b*b - x0*x0 - y0*y0 + 2*h*x0 + 2*k*y0 - k*k - h*h + r*r;
        return descriminant < 0? -1 : -b - Math.sqrt(descriminant);
    };

    this.color = color;
}

function HorizontalLine(lineY, xStart, xEnd, color) {
    this.intersect = function(x0, y0, cos, sin) {
        var dist = (y0-lineY)/sin;
        var x = x0 + dist*cos;
        return (x<=xEnd && x>=xStart)? dist : -1;
    };

    this.color = color;
}

function VerticalLine(lineX, yStart, yEnd, color) {
    this.intersect = function(x0, y0, cos, sin) {
        var dist = (lineX - x0)/cos;
        var y = y0 - dist*sin;
        return (y<=yEnd && y>=yStart)? dist : -1;
    };

    this.color = color;
}

function rgb(r, g, b) {
    return "rgb("+r+","+g+","+b+")";
}

function map(x, inMin, inMax, outMin, outMax) { // the spicy sauce
    return (x - inMin) * (outMax - outMin) / (inMax - inMin) + outMin;
}