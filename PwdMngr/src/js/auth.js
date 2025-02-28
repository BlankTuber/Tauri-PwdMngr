function showError(message) {
    const errorElement = document.getElementById("errorResponse");

    // Set the message and show the element
    errorElement.textContent = message;
    errorElement.classList.add("show");

    // Optional: Play a sound
    const errorSound = new Audio("/assets/error.mp3");
    errorSound.play();

    // Clear any existing timers
    if (window.errorTimer) {
        clearTimeout(window.errorTimer);
    }

    // Automatically hide after 2 seconds
    window.errorTimer = setTimeout(function () {
        errorElement.classList.remove("show");
        errorElement.textContent = "";
    }, 2000);
}
