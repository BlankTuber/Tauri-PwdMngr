function showError(message) {
    const errorElement = document.getElementById("errorResponse");

    // Set the message and show the element
    errorElement.textContent = message;
    errorElement.classList.add("show");

    // Clear any existing timers
    if (window.errorTimer) {
        clearTimeout(window.errorTimer);
    }

    // Automatically hide after 3 seconds
    window.errorTimer = setTimeout(function () {
        errorElement.classList.remove("show");
    }, 3000);
}

function loginUser() {
    const username = document.getElementById("usernameInput").value;
    const password = document.getElementById("passwordInput").value;

    // Validate inputs
    if (!username || !password) {
        showError("Username and password are required!");
        return;
    }

    // Here you would normally make a call to your Tauri backend
    // For demonstration purposes, we'll just show an error
    showError("Login functionality not implemented yet!");
}

function registerUser() {
    const username = document.getElementById("usernameInput").value;
    const password = document.getElementById("passwordInput").value;
    const confirmPassword = document.getElementById(
        "confirmPasswordInput",
    ).value;

    // Validate inputs
    if (!username || !password) {
        showError("Username and password are required!");
        return;
    }

    if (password !== confirmPassword) {
        showError("Passwords do not match!");
        return;
    }

    // Here you would normally make a call to your Tauri backend
    // For demonstration purposes, we'll just show an error
    showError("Registration functionality not implemented yet!");
}
