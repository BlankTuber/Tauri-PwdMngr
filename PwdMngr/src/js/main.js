const { invoke } = window.__TAURI__.core;
const { openUrl } = window.__TAURI__.opener;
const { writeText } = window.__TAURI__.clipboardManager;

if (!sessionStorage.getItem("encKey")) {
    window.location.href = "/login.html";
}

let currentPage = 0;
let totalPages = 1;
let isSearchMode = false;
let lastSearchTerm = "";

document.addEventListener("DOMContentLoaded", function () {
    loadPasswords(1);

    const prevButton = document.querySelector("#pagination button:first-child");
    const nextButton = document.querySelector("#pagination button:last-child");

    prevButton.addEventListener("click", function () {
        if (currentPage > 1) {
            if (isSearchMode) {
                performSearch(currentPage - 1);
            } else {
                loadPasswords(currentPage - 1);
            }
        }
    });

    nextButton.addEventListener("click", function () {
        if (currentPage < totalPages) {
            if (isSearchMode) {
                performSearch(currentPage + 1);
            } else {
                loadPasswords(currentPage + 1);
            }
        }
    });

    // Add search functionality
    const searchInput = document.getElementById("searchInput");
    const searchBtn = document.getElementById("searchBtn");

    searchBtn.addEventListener("click", function () {
        performSearch(1);
    });

    searchInput.addEventListener("keypress", function (event) {
        if (event.key === "Enter") {
            event.preventDefault();
            performSearch(1);
        }
    });

    searchInput.addEventListener("input", function () {
        if (searchInput.value.trim() === "" && isSearchMode) {
            loadPasswords(1);
        }
    });
});

async function loadPasswords(page) {
    try {
        // Reset search mode if active
        if (isSearchMode) {
            isSearchMode = false;
            lastSearchTerm = "";
            document.getElementById("searchInput").value = "";
        }

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
                passwordCard.addEventListener("dblclick", async () => {
                    let url = password.website_url;
                    if (
                        !url.startsWith("http://") &&
                        !url.startsWith("https://")
                    ) {
                        url = "#";
                    }
                    await openUrl(url);
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

        if (currentPage <= totalPages) {
            document.getElementById("pagination").style.display = "flex";
        } else {
            document.getElementById("pagination").style.display = "none";
        }

        document.querySelector(
            "#pagination p",
        ).textContent = `${currentPage} of ${totalPages}`;
    } catch (error) {
        console.error("Failed to load passwords:", error);
    }
}

async function performSearch(page = 1) {
    const searchInput = document.getElementById("searchInput");
    const searchTerm = searchInput.value.trim();

    if (searchTerm === "") {
        isSearchMode = false;
        loadPasswords(1);
        return;
    }

    if (searchTerm === lastSearchTerm && currentPage === page && isSearchMode) {
        return;
    }

    lastSearchTerm = searchTerm;
    isSearchMode = true;

    try {
        const passwordList = document.getElementById("passwordList");
        passwordList.innerHTML = "";

        const encKey = sessionStorage.getItem("encKey");
        const response = await invoke("search_passwords", {
            searchTerm: searchTerm,
            page: page,
            encKey: encKey,
        });

        currentPage = response.page;
        totalPages = response.total_pages;

        // Update pagination display to show search context
        const paginationText = document.querySelector("#pagination p");
        if (response.total) {
            paginationText.textContent = `${response.total} results (Page ${currentPage} of ${totalPages})`;
        } else {
            paginationText.textContent = `0 results (Page 0 of 0)`;
        }

        if (response.passwords.length === 0) {
            const noResults = document.createElement("div");
            noResults.className = "no-results";
            noResults.textContent = `No passwords found matching "${searchTerm}"`;
            passwordList.appendChild(noResults);
            return;
        }

        response.passwords.forEach((password) => {
            const date = new Date(password.updated_at);
            const formattedDate = date.toLocaleDateString();

            const passwordCard = document.createElement("article");
            passwordCard.className = "password-card";
            if (password.website_url) {
                passwordCard.addEventListener("dblclick", async () => {
                    let url = password.website_url;
                    if (
                        !url.startsWith("http://") &&
                        !url.startsWith("https://")
                    ) {
                        url = "https://" + url;
                    }
                    await openUrl(url);
                });
            }

            let websiteDisplay = password.website;
            if (
                searchTerm &&
                password.website
                    .toLowerCase()
                    .includes(searchTerm.toLowerCase())
            ) {
                const regex = new RegExp(`(${escapeRegExp(searchTerm)})`, "gi");
                websiteDisplay = password.website.replace(
                    regex,
                    "<mark>$1</mark>",
                );
            }

            let usernameDisplay = password.username.Ok;
            if (
                searchTerm &&
                password.username.Ok.toLowerCase().includes(
                    searchTerm.toLowerCase(),
                )
            ) {
                const regex = new RegExp(`(${escapeRegExp(searchTerm)})`, "gi");
                usernameDisplay = password.username.Ok.replace(
                    regex,
                    "<mark>$1</mark>",
                );
            }

            let notesDisplay = password.notes || "No notes";
            if (
                searchTerm &&
                password.notes &&
                password.notes.toLowerCase().includes(searchTerm.toLowerCase())
            ) {
                const regex = new RegExp(`(${escapeRegExp(searchTerm)})`, "gi");
                notesDisplay = password.notes.replace(regex, "<mark>$1</mark>");
            }

            passwordCard.innerHTML = `
                <div class="password-header">
                    <p class="website">${websiteDisplay}</p>
                    <p class="username">${usernameDisplay}</p>
                </div>
                <div class="password-body">
                    <p class="notes">${notesDisplay}</p>
                </div>
                <div class="password-footer">
                    <p class="last-updated">
                        <time datetime="${password.updated_at}">${formattedDate}</time>
                    </p>
                    <div class="actions">
                        <button class="copy-btn" onclick="copy('${password.id}-copy')">Copy</button>
                        <button class="edit-btn" onclick="edit('${password.id}')">Edit</button>
                    </div>
                    <input type="hidden" id="${password.id}-copy" value="${password.password.Ok}">
                    <input type="hidden" id="${password.id}" value="${password.id}">
                </div>
            `;

            passwordList.appendChild(passwordCard);
        });
    } catch (error) {
        console.error("Search failed:", error);
        const passwordList = document.getElementById("passwordList");
        passwordList.innerHTML = `<div class="error-message">Search failed: ${error}</div>`;
    }
}

function escapeRegExp(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function logout() {
    sessionStorage.removeItem("encKey");
    invoke("logout_user");
    window.location.href = "/login.html";
}

function newPassword() {
    window.location.href = "/add-password.html";
}

async function copy(id) {
    const input = document.getElementById(id);
    await writeText(input.value);
}

function edit(id) {
    window.location.href = `/edit-password.html?id=${id}`;
}
