'use strict';


const MAP_WIDTH = 500;
const MAP_HEIGHT = 500;
// const window.innerWidth = 500;
// const window.innerHeight = 500;
const PLAYER_RADIUS = 10;
const SHIELD_WIDTH = 4;
const STAR_RADIUS = 3;
const SCALE = 1.5;

var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

canvas.width = window.innerWidth;
canvas.height = window.innerHeight;


// contains:
//      a number called `time` that stores the time that x and y were last valid
//      an array called `stars` each element with an x and y
//      an array called 'players' each with properties x, y, vx, vy
//      an array called `lasers` each with an x, y, facing, and expire
//      an object called 'us' with the same x, y, vx, vy
let gameState; 
let question;

var socket = new WebSocket("ws://ethan.ws/tanks");

let drawLoop;
socket.onopen = function() {
    drawLoop = setInterval(() => {
        draw();
    }, 20);    
};

socket.onclose = die;

socket.onmessage = function(msg) {
    let data = JSON.parse(msg.data);

    if (data.kind == "updateGameState") {
        gameState = data.gameState;
        question = data.question;

        //update the stuffs
        document.getElementById("definition").innerText = question.definition;
        document.getElementById("left").innerText = question.left;
        document.getElementById("right").innerText = question.right;
        updateLeaderboard();
    } else if (data.kind == "kill") {
        die();
    }
};

function die() {
    clearInterval(drawLoop);

    context.fillStyle = "pink";
    context.strokeStyle = "pink";
    context.fillRect(0, 0, window.innerWidth, window.innerHeight);

    document.getElementById("definition").innerText = "YOU";
    document.getElementById("left").innerText = "ARE";
    document.getElementById("right").innerText = "DEAD";
}




function draw() {
    // draw what's in our gameState

    let elapsed = Date.now() - gameState.time; // the time since all of the x's and y's were last valid, hopefully very small

    let us = gameState.us;
    let ourX = us.x + us.vx*elapsed;
    let ourY = us.y - us.vy*elapsed;

    ourX %= MAP_WIDTH;
    if (ourX < 0) { ourX += MAP_WIDTH }
    ourY %= MAP_HEIGHT;
    if (ourY < 0) { ourY += MAP_HEIGHT }

    // update the background
    context.fillStyle = "blue";
    context.strokeStyle = "blue";
    context.fillRect(0, 0, window.innerWidth, window.innerHeight);

    drawLasers(ourX, ourY);
    drawStars(ourX, ourY);
    drawPlayers(ourX, ourY, elapsed);
    drawBorder(ourX, ourY);
}

function drawLasers(ourX, ourY) {
    for (let laser of gameState.lasers) {
        if (laser.expire > Date.now()) {
            let screenX = SCALE*(laser.x - ourX) + window.innerWidth/2;
            let screenY = SCALE*(laser.y - ourY) + window.innerHeight/2;
    
            context.strokeStyle = "lime";
            context.lineWidth = 3;
            context.beginPath();
            context.moveTo(screenX, screenY);
            context.lineTo(screenX + 100000*Math.cos(laser.facing), screenY - 100000*Math.sin(laser.facing));
            context.stroke();
        }
    }
}


function drawPlayers(ourX, ourY, elapsed) {
    for (let player of gameState.players) {
        // figure out where the new position of the player is
        let xNow = player.x + player.vx*elapsed;
        let yNow = player.y - player.vy*elapsed;

        xNow %= MAP_WIDTH;
        if (xNow < 0) { xNow += MAP_WIDTH }
        yNow %= MAP_HEIGHT;
        if (yNow < 0) { yNow += MAP_HEIGHT }
        
        // draw it on the canvas, centered around (ourX, ourY)

        let screenX = SCALE*(xNow - ourX) + window.innerWidth/2;
        let screenY = SCALE*(yNow - ourY) + window.innerHeight/2;

        context.fillStyle = player.color;
        context.strokeStyle = player.color;
        context.beginPath();
        context.arc(screenX, screenY, SCALE*PLAYER_RADIUS, 0, 2*Math.PI);
        context.fill();

        context.lineWidth = 1;
        for (let i=SHIELD_WIDTH; i<=player.shield*SHIELD_WIDTH; i+=SHIELD_WIDTH) { // still repeats `shield` time, but i starts at 1
            // we do monkey stuff here
            context.beginPath();
            context.arc(screenX, screenY, SCALE*(PLAYER_RADIUS+i), 0, 2*Math.PI);
            context.stroke();
        }
    }
}

function drawBorder(ourX, ourY) {
    context.fillStyle = "red";
    context.strokeStyle = "red";  
    context.lineWidth = 3;
    context.strokeRect(SCALE*(0-ourX)+window.innerWidth/2, SCALE*(0-ourY)+window.innerHeight/2, SCALE*MAP_WIDTH, SCALE*MAP_HEIGHT);
}

function drawStars(ourX, ourY) {
    for (let star of gameState.stars) {
        context.fillStyle = "white";
        context.strokeStyle = "white";  
        context.beginPath();      
        context.arc(SCALE*(star.x-ourX)+window.innerWidth/2, SCALE*(star.y-ourY)+window.innerHeight/2, SCALE*STAR_RADIUS, 0, 2*Math.PI);
        context.fill();
    }
}

function updateLeaderboard() {
    let sortable = JSON.parse(JSON.stringify(gameState.players));
    sortable.sort((a, b) => a.shield - b.shield);

    let string = "Leaderboard\n";
    for (let player of sortable) {
        string += player.color + ": " + player.shield + "\n";
    }

    document.getElementById("leaderboard").innerText = string;
}

var lastSentMouseMove = 0;
canvas.addEventListener("mousemove", event => {
    if (Date.now() - lastSentMouseMove > 100) { // our rate limit to stop our server from getting unhappy
        lastSentMouseMove = Date.now();

        // lets calculate the value of newFacing;

        let dx = event.clientX - window.innerWidth/2;
        let dy = window.innerHeight/2 - event.clientY;
        let newFacing = Math.atan2(dy, dx);

        socket.send(JSON.stringify({
            kind: "updateFacing",
            newFacing: newFacing
        }));
    }
});

window.addEventListener("resize", () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;    
});

window.addEventListener("keydown", event => {
    if (event.key == "ArrowLeft" || event.key == "ArrowRight") {
        let guessIsLeft = event.key == "ArrowLeft";

        socket.send(JSON.stringify({
            kind: "guess",
            guessIsLeft: guessIsLeft 
        }))
    } else if (event.key == " ") {
        // IMA FIRIN MY LAZAAAAAAR
        socket.send(JSON.stringify({
            kind: "fire"
        }))
    }
});

function map(x, inMin, inMax, outMin, outMax) { // the spicy sauce
    return (x - inMin) * (outMax - outMin) / (inMax - inMin) + outMin;
}