const { invoke } = window.__TAURI__.core;

if (!sessionStorage.getItem("encKey")) {
    window.location.href = "/login.html";
}


function logout() {
    sessionStorage.removeItem("encKey");
    const response = invoke("logout_user");
    console.log(response);
    window.location.href = "/login.html";
}

function newPassword() {
    window.location.href = "/add-password.html";
}