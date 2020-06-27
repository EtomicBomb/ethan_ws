var socket = new WebSocket("ws://ethan.ws/godset"); // http is boomer stuff

var godset = [];

//socket.onopen = function() {
//    console.log("opened");
//    socket.send("godset"); // the ultimate websocket message
//};

socket.onmessage = function(msg) {
    socket.close();
    console.log(msg.data);
    godset = JSON.parse(msg.data);

    console.log("we have received the glorious godset in all " + godset.length + " of its lines");
};



function clickHandler() {
    // lets do our search
    var targetYearStart = document.getElementById("yearRangeMin").value;
    if (targetYearStart === "") targetYearStart = 1600;

    var targetYearEnd = document.getElementById("yearRangeMax").value;
    if (targetYearEnd === "") targetYearEnd = 2020;

    var targetSocial = document.getElementById("socialChecked").checked;
    var targetPolitical = document.getElementById("politicalChecked").checked;
    var targetEconomic = document.getElementById("economicChecked").checked;

    var keywordValue  = document.getElementById("keywordInput").value;
    var keywordValueLowerCase = keywordValue.toLowerCase();

    var response = "";

    for (var line of godset) {
        var yearsMatch = (targetYearStart <= line.yearStart && line.yearStart <= targetYearEnd)
            || (targetYearStart <= line.yearEnd && line.yearEnd <= targetYearEnd);
        var themesMatch = (targetSocial || !line.social) && (targetPolitical || !line.political) && (targetEconomic || !line.economic);
        var keywordMatch = line.term.toLowerCase().includes(keywordValueLowerCase)
            || line.definition.toLowerCase().includes(keywordValueLowerCase);

        if (yearsMatch && themesMatch && keywordMatch) {
            response += line.term + ": " + line.definition + "\n\n";
        }
    }

    if (response === "") {
        document.getElementById("serverResponse").value = "Couldn't find match";
    } else {
        document.getElementById("serverResponse").value = response;
    }
}

