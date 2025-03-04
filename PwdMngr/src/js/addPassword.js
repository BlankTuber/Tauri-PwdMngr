const { invoke } = window.__TAURI__.core;

function showError(message) {
    const errorElement = document.getElementById("errorResponse");
    errorElement.textContent = message;
    errorElement.classList.add("show");
    if (window.errorTimer) {
        clearTimeout(window.errorTimer);
    }
    window.errorTimer = setTimeout(function () {
        errorElement.classList.remove("show");
    }, 3000);
}

function showSuccess(message) {
    const errorElement = document.getElementById("errorResponse");
    errorElement.style.backgroundColor = "rgba(66, 184, 131, 0.7)";
    errorElement.textContent = message;
    errorElement.classList.add("show");
    if (window.errorTimer) {
        clearTimeout(window.errorTimer);
    }
    window.errorTimer = setTimeout(function () {
        errorElement.classList.remove("show");
        errorElement.style.backgroundColor = "rgba(255, 82, 82, 0.7)";
    }, 3000);
}

async function addNewPassword() {
    if (!sessionStorage.getItem("encKey")) {
        return (location.href = "/login.html");
    }

    const encKey = sessionStorage.getItem("encKey");

    const website = document.getElementById("websiteInput").value.trim();
    const websiteUrl = document.getElementById("websiteUrlInput").value.trim();
    const username = document.getElementById("usernameInput").value.trim();
    const password = document.getElementById("passwordInput").value;
    const notes = document.getElementById("notesInput").value.trim();

    if (!website || !username || !password) {
        return showError("Make sure all required fields are filled in!");
    }

    if (websiteUrl && !isValidUrl(websiteUrl)) {
        return showError("You need to use a valid URL");
    }

    if (
        website.length > 100 ||
        username.length > 40 ||
        password.length > 60 ||
        notes.length > 250
    ) {
        return showError("One or more fields exceed maximum length");
    }

    let params = {
        website,
        username,
        password,
        encKey,
    };

    if (websiteUrl) params.web_uri = websiteUrl;
    if (notes) params.notes = notes;

    const result = await invoke("new_password", params);
    if (result.message) {
        showSuccess(result.message);
    } else {
        showError(result);
    }
}

function isValidUrl(url) {
    try {
        new URL(url);
        return true;
    } catch (e) {
        return false;
    }
}

function cancelForm() {
    location.href = "/";
}

function generatePassword() {
    const passwordInput = document.getElementById("passwordInput");
    const chars =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%&*_+-=.?";
    const length = Math.floor(Math.random() * 9) + 8;
    let password = "";
    for (let i = 0; i < length; i++)
        password += chars.charAt(Math.floor(Math.random() * chars.length));
    passwordInput.value = password;
}

function togglePwd() {
    const passwordInput = document.getElementById("passwordInput");
    passwordInput.type =
        passwordInput.type === "password" ? "text" : "password";
}
