const ws = new WebSocket('ws://127.0.0.1:8080/ws_standard');
const messages = document.getElementById('messages');
const input = document.getElementById('input');
ws.onmessage = (event) => {
    const messageDiv = document.createElement('div');
    messageDiv.className = 'message';
    messageDiv.textContent = event.data;
    messages.appendChild(messageDiv);
    messages.scrollTop = messages.scrollHeight;
};
input.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && input.value) {
        const msg = {
            user_id: 1, // Hardcoded valid ID for testing
            content: input.value
        };
        ws.send(JSON.stringify(msg));
        input.value = '';
    }
});