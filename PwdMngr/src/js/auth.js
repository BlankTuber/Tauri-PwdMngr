// Import Tauri core functionality
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

async function loginUser() {
    const username = document.getElementById("usernameInput").value;
    const password = document.getElementById("passwordInput").value;

    if (!username || !password) {
        showError("Username and password are required!");
        return;
    }

    const submitBtn = document.getElementById("submitBtn");
    submitBtn.disabled = true;
    submitBtn.textContent = "Logging in...";

    try {
        const response = await invoke("login_user", {
            username,
            password,
        });

        if (response.encKey) {
            sessionStorage.setItem("encKey", response.encKey);

            showSuccess(response.message || "Login successful!");

            setTimeout(() => {
                window.location.href = "/index.html";
            }, 1500);
        } else {
            showError("Login successful but failed to get encryption key");
        }
    } catch (error) {
        showError(error.toString());
    } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = "Login";
    }
}

async function registerUser() {
    const username = document.getElementById("usernameInput").value;
    const password = document.getElementById("passwordInput").value;
    const confirmPassword = document.getElementById("confirmPasswordInput").value;

    if (!username || !password) {
        showError("Username and password are required!");
        return;
    }

    if (password !== confirmPassword) {
        showError("Passwords do not match!");
        return;
    }

    const submitBtn = document.getElementById("submitBtn");
    submitBtn.disabled = true;
    submitBtn.textContent = "Registering...";

    try {
        const response = await invoke("register_user", {
            username,
            password,
            confirmPassword,
        });

        if (response.encKey) {
            sessionStorage.setItem("encKey", response.encKey);

            showSuccess(response.message || "Registration successful!");

            setTimeout(() => {
                window.location.href = "/index.html";
            }, 1500);
        } else {
            showError("Registration successful but failed to get encryption key");
        }
    } catch (error) {
        showError(error.toString());
    } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = "Register";
    }
}
