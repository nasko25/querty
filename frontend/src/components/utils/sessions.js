export function generateSession() {
    var res = '';
    var alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for ( var i = 0; i < 20; i++ ) {
        res += alphabet.charAt(Math.floor(Math.random() * alphabet.length));
    }
    return res;
}

export function setCookie() {
    const expr_date = new Date();
    expr_date.setTime(expr_date.getTime() + (60 *24*60*60*1000)); // set the expiration date 60 days from now
    let expires = "expires="+ expr_date.toUTCString();
    document.cookie = "session=" + generateSession() + ";" + expires + ";path=/";
}

export function getSession() {
    let cookies = document.cookie.split(';');
        for(let i = 0; i < cookies.length; i++) {
            let cookie = cookies[i];
            while (cookie.charAt(0) == ' ') {
                cookie = cookie.substring(1);
            }
            if (cookie.indexOf("session") == 0) {
                return cookie.substring("session".length, cookie.length);
            }
        }
    return "";
}

export function checkSession() {
    return getSession() !== "";
}
