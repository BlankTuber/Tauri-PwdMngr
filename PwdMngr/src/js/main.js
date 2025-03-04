const { invoke } = window.__TAURI__.core;

if (!sessionStorage.getItem("encKey")) {
    window.location.href = "/login.html";
}

let currentPage = 1;
let totalPages = 1;

document.addEventListener("DOMContentLoaded", function () {
    loadPasswords(1);
});

async function loadPasswords(page) {
    try {
        const passwordList = document.getElementById("passwordList");
        passwordList.innerHTML = "";

        const encKey = sessionStorage.getItem("encKey");
        const response = await invoke("get_passwords", { page, encKey });

        currentPage = response.page;
        totalPages = response.total_pages;

        response.passwords.forEach((password) => {
            const date = new Date(password.updated_at);
            const formattedDate = date.toLocaleDateString();

            const passwordCard = document.createElement("article");
            passwordCard.className = "password-card";
            if (password.website_url) {
                console.log(password.website_url);
                passwordCard.addEventListener("click", () => {
                    console.log("Website Found!");
                    location.href = password.website_url;
                });
            }
            passwordCard.innerHTML = `
                <div class="password-header">
                    <p class="website">${password.website}</p>
                    <p class="username">${password.username.Ok}</p>
                </div>
                <div class="password-body">
                    <p class="notes">${password.notes || "No notes"}</p>
                </div>
                <div class="password-footer">
                    <p class="last-updated">
                        <time datetime="${
                            password.updated_at
                        }">${formattedDate}</time>
                    </p>
                    <div class="actions">
                        <button class="copy-btn" onclick="copy('${
                            password.id
                        }-copy')">Copy</button>
                        <button class="edit-btn" onclick="edit('${
                            password.id
                        }')">Edit</button>
                    </div>
                    <input type="hidden" id="${password.id}-copy" value="${
                password.password.Ok
            }">
                    <input type="hidden" id="${password.id}" value="${
                password.id
            }">
                </div>
            `;

            passwordList.appendChild(passwordCard);
        });

        document.querySelector(
            "#pagination p",
        ).textContent = `${currentPage} of ${totalPages}`;
    } catch (error) {
        console.error("Failed to load passwords:", error);
    }
}

document.addEventListener("DOMContentLoaded", function () {
    const prevButton = document.querySelector("#pagination button:first-child");
    const nextButton = document.querySelector("#pagination button:last-child");

    prevButton.addEventListener("click", function () {
        if (currentPage > 1) {
            loadPasswords(currentPage - 1);
        }
    });

    nextButton.addEventListener("click", function () {
        if (currentPage < totalPages) {
            loadPasswords(currentPage + 1);
        }
    });
});

function logout() {
    sessionStorage.removeItem("encKey");
    invoke("logout_user");
    window.location.href = "/login.html";
}

function newPassword() {
    window.location.href = "/add-password.html";
}
