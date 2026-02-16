const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const host = window.location.host;

const loginOverlay = document.getElementById('login-overlay');
const loginButton = document.getElementById('login-button');
const registerButton = document.getElementById('register-button');
const usernameInput = document.getElementById('username');
const passwordInput = document.getElementById('password');
const loginError = document.getElementById('login-error');
const loginSuccess = document.getElementById('login-success');
const chatBox = document.getElementById('chat-box');
const messages = document.getElementById('messages');
const input = document.getElementById('input');
const logoutButton = document.getElementById('logout-button');

let ws;
let currentUsername = "";

let currentUserId = 0;

loginButton.addEventListener('click', async () => {
    const username = usernameInput.value;
    const password = passwordInput.value;

    if (!username || !password) {
        showError("Username and password are required");
        return;
    }

    try {
        const response = await fetch('/api/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                username: username,
                password: password,
            }),
        });

        const data = await response.json();

        if (response.ok) {
            currentUsername = username;
            currentUserId = data.user_id;
            startChat();
        } else {
            showError(data || "Invalid credentials");
        }
    } catch (err) {
        showError("Could not connect to server");
    }
});

registerButton.addEventListener('click', async () => {
    const username = usernameInput.value;
    const password = passwordInput.value;

    if (!username || !password) {
        showError("Username and password are required");
        return;
    }

    try {
        const response = await fetch('/api/register', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                username: username,
                password: password,
            }),
        });

        const data = await response.json();

        if (response.ok) {
            currentUsername = username;
            currentUserId = data.user_id;
            showSuccess(data.message || "Registration successful! You can now login.");
            loginError.classList.add('hidden');
        } else {
            showError(data || "Registration failed");
            loginSuccess.classList.add('hidden');
        }
    } catch (err) {
        showError("Could not connect to server");
    }
});

function showError(msg) {
    loginError.textContent = msg;
    loginError.classList.remove('hidden');
    loginSuccess.classList.add('hidden');
}

function showSuccess(msg) {
    loginSuccess.textContent = msg;
    loginSuccess.classList.remove('hidden');
    loginError.classList.add('hidden');
}

async function load_history(limit) {
    try {
        const response = await fetch('/api/get_chat_history?limit=' + limit, {
            method: 'GET',
        });

        const data = await response.json();

        if (response.ok) {
            data.forEach(current_message => {
                const messageDiv = document.createElement('div');
                messageDiv.className = 'message';
                messageDiv.textContent = `< ${current_message.username} > - ${current_message.content}`;
                messages.appendChild(messageDiv);
                messages.scrollTop = messages.scrollHeight;
            });
        } else {
            showError(data || "Unable to retrieve the chat_history");
        }
    } catch (err) {
        showError("Could not connect with server");
    }
}

function startChat() {
    loginOverlay.classList.add('hidden');
    chatBox.classList.remove('hidden');
    input.focus();

    ws = new WebSocket(`${protocol}//${host}/ws`);

    load_history(50);

    ws.onmessage = (event) => {
        const messageDiv = document.createElement('div');
        messageDiv.className = 'message';
        let data = JSON.parse(event.data);
        messageDiv.textContent = `< ${data.username} > - ${data.content}`;
        messages.appendChild(messageDiv);
        messages.scrollTop = messages.scrollHeight;
    };
}

input.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && input.value && ws) {
        const msg = {
            user_id: currentUserId,
            username: currentUsername,
            content: input.value
        };
        ws.send(JSON.stringify(msg));
        input.value = '';
    }
});

logoutButton.addEventListener('click', async () => {
    try {
        await fetch('/api/logout', { method: 'POST' });
        window.location.reload();
    } catch (err) {
        console.error("Logout failed", err);
        window.location.reload();
    }
});

// Check for existing session
async function checkSession() {
    try {
        const response = await fetch('/api/me');
        if (response.ok) {
            const data = await response.json();
            currentUsername = data.username;
            currentUserId = data.id;
            startChat();
        } else {
            loginOverlay.classList.remove('hidden');
        }
    } catch (err) {
        console.log("No active session found");
        loginOverlay.classList.remove('hidden');
    }
}

checkSession();
