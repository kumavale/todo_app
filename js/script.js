// Adjust input field
function auto_height(argObj) {
    argObj.style.height = "10px";
    var wSclollHeight = parseInt(argObj.scrollHeight);
    var wLineH = parseInt(argObj.style.lineHeight.replace(/px/, ''));
    if (wSclollHeight < (wLineH * 2)) {
        wSclollHeight = (wLineH * 2);
    }
    argObj.style.height = wSclollHeight + "px";
}

