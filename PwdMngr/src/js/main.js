const { invoke } = window.__TAURI__.core;

if (!sessionStorage.getItem("encKey")) {
    window.location.href = "/login.html";
}
