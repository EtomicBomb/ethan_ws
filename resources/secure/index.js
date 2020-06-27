const socket = new WebSocket("ws://ethan.ws/secure");

function onSubmit() {
    let text = document.getElementById("passwordField").value;

    console.log(text);

    socket.send(text);

    document.getElementById("securityLabel").innerText = "Status: " + randomize();

    setTimeout(() => {
        document.getElementById("securityLabel").innerText = "Status: ";
    }, 2000);
}



const RESPONSES = [
    "Secure",
    "Insecure",
    "Bad",
    "Pretty Good",
    "Ask Again Later",
    "The Spirits Are Uncertain",
    "Tasty",
    "Honestly like 6/10, not to good, not too bad",
    "You should probably change it",
    "Fine",
    "Hmm... Thinking...",
    "Wow... That's the best password I've ever seen",
];

function randomize() {
    let index = RESPONSES.length * Math.random();
    return RESPONSES[index|0];
}