const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const host = window.location.host;

const loginOverlay = document.getElementById('login-overlay');
const loginButton = document.getElementById('login-button');
const usernameInput = document.getElementById('username');
const passwordInput = document.getElementById('password');
const loginError = document.getElementById('login-error');
const chatBox = document.getElementById('chat-box');
const messages = document.getElementById('messages');
const input = document.getElementById('input');

let ws;
let currentUsername = "";

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

        if (response.ok) {
            currentUsername = username;
            startChat();
        } else {
            const data = await response.json();
            showError(data || "Invalid credentials");
        }
    } catch (err) {
        showError("Could not connect to server");
    }
});

function showError(msg) {
    loginError.textContent = msg;
    loginError.classList.remove('hidden');
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
        messageDiv.textContent = `< ${data.user_id} > - ${data.content}`;
        messages.appendChild(messageDiv);
        messages.scrollTop = messages.scrollHeight;
    };
}

input.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && input.value && ws) {
        const msg = {
            user_name: currentUsername,
            content: input.value
        };
        ws.send(JSON.stringify(msg));
        input.value = '';
    }
});
