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

function startChat() {
    loginOverlay.classList.add('hidden');
    chatBox.classList.remove('hidden');
    input.focus();

    ws = new WebSocket(`${protocol}//${host}/ws`);

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
