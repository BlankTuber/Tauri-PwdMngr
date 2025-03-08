const { invoke } = window.__TAURI__.core;

let passwordId = "";

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
    errorElement.textContent = message;
    errorElement.classList.add("show", "success");
    if (window.errorTimer) {
        clearTimeout(window.errorTimer);
    }
    window.errorTimer = setTimeout(function () {
        errorElement.classList.remove("show", "success");
    }, 3000);
}

document.addEventListener("DOMContentLoaded", async function () {
    if (!sessionStorage.getItem("encKey")) {
        return (location.href = "/login.html");
    }

    // Get password ID from URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    passwordId = urlParams.get("id");

    if (!passwordId) {
        showError("No password ID provided");
        setTimeout(() => {
            location.href = "/";
        }, 1500);
        return;
    }

    // Load password data
    await loadPasswordData();

    // Set up event listeners
    const passwordInput = document.getElementById("passwordInput");
    if (passwordInput) {
        passwordInput.addEventListener("input", function () {
            checkPasswordStrength(this.value);
        });
    }
});

async function loadPasswordData() {
    try {
        const encKey = sessionStorage.getItem("encKey");
        const response = await invoke("get_password_details", {
            id: passwordId,
            encKey,
        });

        if (response) {
            document.getElementById("websiteInput").value = response.website;
            if (response.website_url) {
                document.getElementById("websiteUrlInput").value =
                    response.website_url;
            }
            document.getElementById("usernameInput").value =
                response.username.Ok;
            document.getElementById("passwordInput").value =
                response.password.Ok;

            if (response.notes) {
                document.getElementById("notesInput").value = response.notes;
            }

            // Check password strength
            checkPasswordStrength(response.password.Ok);
        } else {
            showError("Failed to load password details");
        }
    } catch (error) {
        showError(error.toString());
        console.error("Failed to load password details:", error);
    }
}

async function updatePassword() {
    if (!sessionStorage.getItem("encKey")) {
        return (location.href = "/login.html");
    }

    const encKey = sessionStorage.getItem("encKey");

    const website = document.getElementById("websiteInput").value.trim();
    const website_url = document.getElementById("websiteUrlInput").value.trim();
    const username = document.getElementById("usernameInput").value.trim();
    const password = document.getElementById("passwordInput").value;
    const notes = document.getElementById("notesInput").value.trim();

    if (!website || !username || !password) {
        return showError("Make sure all required fields are filled in!");
    }

    if (website_url && !isValidUrl(website_url)) {
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
        id: passwordId,
        website,
        username,
        password,
        encKey,
    };

    if (website_url) params.websiteUrl = website_url;
    if (notes) params.notes = notes;

    try {
        const result = await invoke("update_password", params);
        if (result.message) {
            showSuccess(result.message);
            setTimeout(() => {
                location.href = "/";
            }, 750);
        } else {
            showError(result);
        }
    } catch (error) {
        showError(error.toString());
    }
}

async function deletePassword() {
    // Create modal to confirm deletion
    const modal = document.createElement("div");
    modal.className = "delete-modal";
    modal.innerHTML = `
        <div class="modal-content">
            <h3>Confirm Deletion</h3>
            <p>Are you sure you want to delete this password? This action cannot be undone.</p>
            <div class="modal-actions">
                <button class="cancel-delete">Cancel</button>
                <button class="confirm-delete">Delete</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.classList.add("show");

    // Set up event listeners for modal buttons
    document.querySelector(".cancel-delete").addEventListener("click", () => {
        modal.classList.remove("show");
        setTimeout(() => {
            document.body.removeChild(modal);
        }, 300);
    });

    document
        .querySelector(".confirm-delete")
        .addEventListener("click", async () => {
            try {
                const result = await invoke("delete_password", {
                    id: passwordId,
                });
                if (result.message) {
                    modal.classList.remove("show");
                    setTimeout(() => {
                        document.body.removeChild(modal);
                        showSuccess(result.message);
                        setTimeout(() => {
                            location.href = "/";
                        }, 750);
                    }, 300);
                } else {
                    showError(result);
                }
            } catch (error) {
                showError(error.toString());
            }
        });
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

function checkPasswordStrength(password) {
    const strengthIndicator = document.getElementById("passwordStrength");
    const strengthLabel = document.getElementById("strengthLabel");

    if (!password) {
        strengthIndicator.style.width = "0%";
        strengthIndicator.style.backgroundColor = "#ff6b6b";
        strengthLabel.textContent = "Password Strength";
        return;
    }

    let strength = 0;

    if (password.length >= 8) {
        strength += 25;
    } else if (password.length >= 6) {
        strength += 10;
    }

    if (password.match(/[a-z]/)) strength += 10;
    if (password.match(/[A-Z]/)) strength += 15;
    if (password.match(/[0-9]/)) strength += 15;
    if (password.match(/[^a-zA-Z0-9]/)) strength += 20;

    const uniqueChars = new Set(password.split("")).size;
    strength += Math.min(15, uniqueChars * 2);

    strength = Math.min(100, strength);

    strengthIndicator.style.width = `${strength}%`;

    if (strength < 40) {
        strengthIndicator.style.backgroundColor = "#ff6b6b";
        strengthLabel.textContent = "Weak Password";
    } else if (strength < 70) {
        strengthIndicator.style.backgroundColor = "#ffdd57";
        strengthLabel.textContent = "Medium Password";
    } else {
        strengthIndicator.style.backgroundColor = "#48c774";
        strengthLabel.textContent = "Strong Password";
    }

    return strength;
}

function generatePassword() {
    const passwordInput = document.getElementById("passwordInput");
    const chars =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%&*_+-=.?";
    const length = Math.floor(Math.random() * 9) + 8;
    let password = "";

    password += chars.match(/[a-z]/)[0];
    password += chars.match(/[A-Z]/)[0];
    password += chars.match(/[0-9]/)[0];
    password += chars.match(/[^a-zA-Z0-9]/)[0];

    for (let i = 4; i < length; i++)
        password += chars.charAt(Math.floor(Math.random() * chars.length));

    password = password
        .split("")
        .sort(() => 0.5 - Math.random())
        .join("");

    passwordInput.value = password;

    checkPasswordStrength(password);
}

function togglePwd() {
    const passwordInput = document.getElementById("passwordInput");
    passwordInput.type =
        passwordInput.type === "password" ? "text" : "password";
}
