const { invoke } = window.__TAURI__.core;

// Store passwords data
let allPasswords = [];

document.addEventListener("DOMContentLoaded", function () {
    // Check authentication
    if (!sessionStorage.getItem("encKey")) {
        window.location.href = "/login.html";
        return;
    }

    // Load passwords for export
    loadAllPasswordsForExport();

    // Set up file input change listener for import
    const fileInput = document.getElementById("importFile");
    if (fileInput) {
        fileInput.addEventListener("change", handleFileSelection);
    }
});

// Tab switching function
function switchTab(tab) {
    // Hide all tab contents
    document.querySelectorAll(".ei-tab-content").forEach((content) => {
        content.classList.remove("active");
    });

    // Show selected tab content
    document.getElementById(`${tab}Tab`).classList.add("active");

    // Update tab button states
    document.querySelectorAll(".ei-tab-btn").forEach((button) => {
        button.classList.remove("active");
    });

    // Find the clicked button and make it active
    const clickedButton = document.querySelector(
        `.ei-tab-btn[onclick="switchTab('${tab}')"]`,
    );
    if (clickedButton) {
        clickedButton.classList.add("active");
    }
}

// Load all passwords for export
async function loadAllPasswordsForExport() {
    try {
        const encKey = sessionStorage.getItem("encKey");
        if (!encKey) {
            throw new Error("Authentication key not found");
        }

        // Show loading indicator
        const passwordList = document.getElementById("passwordList");
        passwordList.innerHTML =
            '<div class="ei-loading">Loading passwords...</div>';

        const response = await invoke("get_all_passwords_for_export", {
            encKey,
        });

        if (!response || !response.passwords) {
            throw new Error("Invalid response from server");
        }

        allPasswords = response.passwords;
        renderPasswordList(allPasswords);
    } catch (error) {
        console.error("Failed to load passwords:", error);
        showStatus(error.toString(), "error");

        // Clear password list with error message
        const passwordList = document.getElementById("passwordList");
        passwordList.innerHTML = `<div class="ei-error">Error loading passwords: ${error.toString()}</div>`;
    }
}

// Render password list with checkboxes
function renderPasswordList(passwords) {
    const passwordList = document.getElementById("passwordList");
    passwordList.innerHTML = "";

    if (!passwords || passwords.length === 0) {
        passwordList.innerHTML =
            '<div class="ei-no-passwords">No passwords found</div>';
        return;
    }

    passwords.forEach((password) => {
        // Safe date formatting
        let formattedDate = "Unknown";
        try {
            if (password.updated_at) {
                const date = new Date(password.updated_at);
                formattedDate = date.toLocaleDateString();
            }
        } catch (e) {
            console.warn("Date formatting error:", e);
        }

        const passwordItem = document.createElement("div");
        passwordItem.className = "ei-password-item";

        // Create HTML with safe defaults
        passwordItem.innerHTML = `
            <div class="ei-checkbox-col">
                <input type="checkbox" class="ei-checkbox" data-id="${
                    password.id || ""
                }" checked>
            </div>
            <div class="ei-website-col">${escapeHtml(
                password.website || "Unknown",
            )}</div>
            <div class="ei-username-col">${escapeHtml(
                password.username || "Unknown",
            )}</div>
            <div class="ei-date-col">${formattedDate}</div>
        `;

        passwordList.appendChild(passwordItem);
    });
}

// Helper function to escape HTML
function escapeHtml(unsafe) {
    return String(unsafe)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

// Select all passwords
function selectAll() {
    document.querySelectorAll(".ei-checkbox").forEach((checkbox) => {
        checkbox.checked = true;
    });
}

// Deselect all passwords
function deselectAll() {
    document.querySelectorAll(".ei-checkbox").forEach((checkbox) => {
        checkbox.checked = false;
    });
}

// Export selected passwords
async function exportPasswords() {
    try {
        const selectedIds = Array.from(
            document.querySelectorAll(".ei-checkbox:checked"),
        )
            .map((checkbox) => checkbox.getAttribute("data-id"))
            .filter((id) => id); // Filter out any null/empty IDs

        if (selectedIds.length === 0) {
            showStatus("No passwords selected for export", "error");
            return;
        }

        // Get export format
        const format = document.getElementById("exportFormat").value;

        // Show loading status
        showStatus(`Preparing export... Please wait.`, "success");

        // Get decrypted password data from backend
        const encKey = sessionStorage.getItem("encKey");
        const response = await invoke("prepare_passwords_for_export", {
            encKey,
            selectedIds,
        });

        if (!response.success || !response.data) {
            throw new Error("Failed to prepare passwords for export");
        }

        // Generate export data
        let exportData;
        let filename;
        let mimeType;

        if (format === "json") {
            // Format for JSON export
            exportData = JSON.stringify(response.data, null, 2);
            filename = "passwords_export.json";
            mimeType = "application/json";
        } else if (format === "csv") {
            // Format for CSV export
            exportData = convertToCSV(response.data);
            filename = "passwords_export.csv";
            mimeType = "text/csv";
        } else {
            throw new Error("Invalid export format");
        }

        // Create a Blob and download link
        const blob = new Blob([exportData], { type: mimeType });
        const url = URL.createObjectURL(blob);

        // Create download link
        const link = document.createElement("a");
        link.href = url;
        link.download = filename;
        link.style.display = "none";

        // Add to body, click, and remove
        document.body.appendChild(link);
        link.click();

        // Clean up and show a more detailed success message
        setTimeout(() => {
            URL.revokeObjectURL(url);
            document.body.removeChild(link);

            // Show success with download location information
            const downloadsFolder = getDefaultDownloadsFolder();
            showStatus(
                `Successfully exported ${response.data.length} passwords to ${filename}. 
                       File saved to your downloads folder (${downloadsFolder})`,
                "success",
            );
        }, 100);
    } catch (error) {
        console.error("Export error:", error);
        showStatus(`Export failed: ${error}`, "error");
    }
}

// Helper function to get the likely downloads folder based on platform
function getDefaultDownloadsFolder() {
    // Detect OS to provide more accurate information
    const userAgent = navigator.userAgent.toLowerCase();

    if (userAgent.indexOf("win") !== -1) {
        return "C:\\Users\\[YourUsername]\\Downloads";
    } else if (userAgent.indexOf("mac") !== -1) {
        return "~/Downloads";
    } else if (userAgent.indexOf("linux") !== -1) {
        return "~/Downloads";
    } else {
        return "your Downloads folder";
    }
}

// Convert array of objects to CSV
function convertToCSV(data) {
    if (!data || !data.length) return "";

    // Get headers from the first object
    const headers = ["website", "username", "password", "website_url", "notes"];

    // Create CSV header row
    const csvRows = [headers.join(",")];

    // Create data rows
    data.forEach((item) => {
        const row = headers.map((header) => {
            let value = item[header] || "";

            // Properly escape CSV values
            value = String(value).replace(/"/g, '""');

            // Wrap in quotes if contains special characters
            if (
                value.includes(",") ||
                value.includes('"') ||
                value.includes("\n")
            ) {
                value = `"${value}"`;
            }

            return value;
        });

        csvRows.push(row.join(","));
    });

    return csvRows.join("\n");
}

// Show status message
function showStatus(message, type) {
    const statusEl = document.getElementById("statusMessage");
    if (!statusEl) return;

    statusEl.textContent = message;
    statusEl.className = "ei-status " + type; // 'error' or 'success'

    // Auto-hide after 5 seconds
    setTimeout(() => {
        statusEl.className = "ei-status";
    }, 5000);
}

// Handle file selection for import
async function handleFileSelection(event) {
    const file = event.target.files[0];
    if (!file) return;

    const previewEl = document.getElementById("importPreview");
    previewEl.innerHTML = `<div class="ei-loading">Analyzing file: ${escapeHtml(
        file.name,
    )}...</div>`;

    const reader = new FileReader();

    reader.onload = function (e) {
        try {
            const fileContent = e.target.result;
            const fileExtension = file.name.split(".").pop().toLowerCase();

            let passwords = [];

            if (fileExtension === "json") {
                // Parse JSON file
                passwords = parseJSONData(fileContent);
            } else if (fileExtension === "csv") {
                // Parse CSV file
                passwords = parseCSVData(fileContent);
            } else {
                throw new Error(
                    "Unsupported file format. Please use JSON or CSV files.",
                );
            }

            // Store passwords data in a data attribute for import
            previewEl.setAttribute("data-passwords", JSON.stringify(passwords));

            // Render preview
            renderImportPreview(passwords);
        } catch (error) {
            console.error("File parsing error:", error);
            previewEl.innerHTML = `<div class="ei-error">Error: ${error.message}</div>`;
        }
    };

    reader.onerror = function () {
        previewEl.innerHTML = `<div class="ei-error">Error reading file</div>`;
    };

    reader.readAsText(file);
}

// Parse JSON data safely
function parseJSONData(content) {
    try {
        const data = JSON.parse(content);

        // Validate it's an array
        if (!Array.isArray(data)) {
            throw new Error(
                "Invalid JSON format. Expected an array of password objects.",
            );
        }

        // Validate each object has required fields
        const validPasswords = data.filter((pwd) => {
            return (
                pwd &&
                typeof pwd === "object" &&
                pwd.website &&
                pwd.username &&
                pwd.password
            );
        });

        if (validPasswords.length === 0) {
            throw new Error(
                "No valid password entries found in the JSON file.",
            );
        }

        return validPasswords;
    } catch (error) {
        if (error instanceof SyntaxError) {
            throw new Error(`Invalid JSON format: ${error.message}`);
        }
        throw error;
    }
}

// Parse CSV data safely
function parseCSVData(content) {
    try {
        // Split by lines
        const lines = content.split(/\r?\n/).filter((line) => line.trim());

        if (lines.length < 2) {
            throw new Error(
                "CSV file must contain a header row and at least one data row.",
            );
        }

        // Parse header
        const header = parseCSVLine(lines[0]);

        // Check required headers
        const requiredFields = ["website", "username", "password"];
        for (const field of requiredFields) {
            if (!header.includes(field)) {
                throw new Error(`CSV is missing required column: ${field}`);
            }
        }

        // Parse rows
        const passwords = [];

        for (let i = 1; i < lines.length; i++) {
            if (!lines[i].trim()) continue;

            const values = parseCSVLine(lines[i]);
            if (values.length !== header.length) {
                console.warn(`Skipping row ${i + 1}: Column count mismatch`);
                continue;
            }

            const entry = {};
            header.forEach((column, index) => {
                entry[column.trim()] = values[index];
            });

            // Check required fields
            if (entry.website && entry.username && entry.password) {
                passwords.push(entry);
            } else {
                console.warn(`Skipping row ${i + 1}: Missing required fields`);
            }
        }

        if (passwords.length === 0) {
            throw new Error("No valid password entries found in the CSV file.");
        }

        return passwords;
    } catch (error) {
        throw error;
    }
}

// Parse a CSV line handling quoted values correctly
function parseCSVLine(line) {
    const result = [];
    let currentValue = "";
    let inQuotes = false;

    for (let i = 0; i < line.length; i++) {
        const char = line[i];

        if (char === '"') {
            // Look ahead for another quote (escaped quote)
            if (i < line.length - 1 && line[i + 1] === '"') {
                currentValue += '"';
                i++; // Skip the next quote
            } else {
                // Toggle quote mode
                inQuotes = !inQuotes;
            }
        } else if (char === "," && !inQuotes) {
            // End of field
            result.push(currentValue);
            currentValue = "";
        } else {
            // Regular character
            currentValue += char;
        }
    }

    // Don't forget the last field
    result.push(currentValue);

    return result;
}

// Render import preview
function renderImportPreview(passwords) {
    const previewEl = document.getElementById("importPreview");

    if (!passwords || passwords.length === 0) {
        previewEl.innerHTML =
            '<div class="ei-no-passwords">No valid passwords found in the file</div>';
        return;
    }

    // Display preview table
    let html = `
        <h3>Found ${passwords.length} passwords to import</h3>
        <div class="ei-preview-table">
            <div class="ei-preview-header">
                <div>Website</div>
                <div>Username</div>
                <div>Password</div>
            </div>
    `;

    // Limit preview to first 10 items
    const previewLimit = Math.min(passwords.length, 10);

    for (let i = 0; i < previewLimit; i++) {
        const pwd = passwords[i];
        html += `
            <div class="ei-preview-row">
                <div>${escapeHtml(pwd.website || "N/A")}</div>
                <div>${escapeHtml(pwd.username || "N/A")}</div>
                <div>${pwd.password ? "********" : "N/A"}</div>
            </div>
        `;
    }

    if (passwords.length > previewLimit) {
        html += `<div class="ei-preview-more">...and ${
            passwords.length - previewLimit
        } more</div>`;
    }

    html += "</div>";

    previewEl.innerHTML = html;
}

// Import passwords
async function importPasswords() {
    const previewEl = document.getElementById("importPreview");
    const passwordsData = previewEl.getAttribute("data-passwords");

    if (!passwordsData) {
        showStatus(
            "No passwords to import. Please select a file first.",
            "error",
        );
        return;
    }

    let passwords;
    try {
        passwords = JSON.parse(passwordsData);
    } catch (e) {
        showStatus("Failed to parse password data", "error");
        return;
    }

    if (!passwords || !Array.isArray(passwords) || passwords.length === 0) {
        showStatus("No valid passwords found to import.", "error");
        return;
    }

    // Confirm import
    if (
        !confirm(
            `Are you sure you want to import ${passwords.length} passwords?`,
        )
    ) {
        return;
    }

    const encKey = sessionStorage.getItem("encKey");
    if (!encKey) {
        showStatus("Authentication error. Please log in again.", "error");
        setTimeout(() => {
            window.location.href = "/login.html";
        }, 1500);
        return;
    }

    // Disable import button
    const importBtn = document.getElementById("importBtn");
    if (importBtn) {
        importBtn.disabled = true;
        importBtn.textContent = "Importing...";
    }

    // Show progress
    showStatus(`Importing passwords... Please wait.`, "success");

    // Call backend to handle import
    try {
        const result = await invoke("import_passwords_from_data", {
            passwordsData: passwords,
            encKey,
        });

        if (result.success) {
            showStatus(
                `Successfully imported ${result.success_count} passwords${
                    result.error_count > 0
                        ? ` with ${result.error_count} errors`
                        : ""
                }`,
                "success",
            );
        } else {
            throw new Error(result.message || "Import failed");
        }
    } catch (error) {
        console.error("Import error:", error);
        showStatus(`Import failed: ${error}`, "error");
    } finally {
        // Re-enable import button
        if (importBtn) {
            importBtn.disabled = false;
            importBtn.textContent = "Import Passwords";
        }

        // Reset file input
        document.getElementById("importFile").value = "";
        previewEl.innerHTML = "";
        previewEl.removeAttribute("data-passwords");

        // Refresh password list after import
        setTimeout(() => {
            loadAllPasswordsForExport();
        }, 1500);
    }
}
