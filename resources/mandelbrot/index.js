const BAILOUT = 500;
const BAILOUT_EQUIVALENT = 20;

const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");

const CANVAS_STEP = 1;


canvas.width = window.innerWidth;
canvas.height = window.innerHeight;

draw();


function draw() {
    let minX = -2;
    let maxX = 2;
    let minY  = -2;
    let maxY = 2;

    let dMathX = map(CANVAS_STEP, 0, canvas.clientHeight, 0, maxX-minX);
    let dMathY = map(CANVAS_STEP, 0, canvas.clientHeight, 0, maxY-minY);

    console.log(dMathX);
    console.log(dMathY);

    let rows = [];

    let min = Infinity;
    let max = 0;

    for (let canvasY=0, mathY=minY; canvasY<canvas.clientHeight; canvasY+=CANVAS_STEP, mathY+=dMathY) {
        let cells = [];

        for (let canvasX=0, mathX=minX; canvasX<canvas.clientWidth; canvasX+=CANVAS_STEP, mathX+=dMathX) {
            let color = escape(mathX, mathY);

            min = Math.min(min, color);
            max = Math.max(max, color);

            cells.push(color);
        }

        rows.push(cells);
    }

    if (max == BAILOUT) max = BAILOUT_EQUIVALENT;

    context.strokeStyle = "#00000000";

    for (let y=0; y<rows.length; y++) {
        let screenY = y * CANVAS_STEP;

        for (let x=0; x<rows[0].length; x++) {
            let screenX = x * CANVAS_STEP;

            context.fillStyle = colorize(rows[y][x], min, max);

            context.fillRect(screenX, screenY, CANVAS_STEP, CANVAS_STEP);
        }
    }
}

function colorize(escape, min, max) {
    let value = map(escape, min, max, 255, 0);

    return rgb(value, value, value); 
}

function rgb(r, g, b) {
    return "rgb("+r+","+g+","+b+")";
}


function escape(a, b) {
    let count = 0;

    let z_real = 0;
    let z_imag = 0;

    let c_real = a;
    let c_imag = b;

    while (Math.hypot(z_real, z_imag) < 2 && count < BAILOUT) {

        let new_real = z_real*z_real - z_imag*z_imag + c_real;
        z_imag = 2*z_real*z_imag + c_imag;
        z_real = new_real;

        count += 1;
    }


    return count;
}

function map(x, in_min, in_max, out_min, out_max) {
    // https://www.arduino.cc/reference/en/language/functions/math/map/
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
