const MAX_HISTORY_SIZE = 60;
const socket = new WebSocket("ws://localhost:3030/chat");

socket.onopen = (_) => {
    console.debug("Connection established");    
};

socket.onmessage = (event) => {
    const newMessage = JSON.parse(event.data);

    // Append new message
    const messageContainerEl = document.createElement('div');
    messageContainerEl.id = 'message';
    messageContainerEl.innerHTML = `<b>> ${newMessage['username']}</b>: ${newMessage['message']}`;

    const chatContainerEl = document.getElementById('chat');
    chatContainerEl.prepend(messageContainerEl);

    // Clear older messages to keep under the maximum number
    const historyMessages = chatContainerEl.getElementsByTagName('div');
    while (historyMessages.length > MAX_HISTORY_SIZE) {
        const messageToBeRemoved = historyMessages[historyMessages.length - 1];
        chatContainerEl.removeChild(messageToBeRemoved);
    }
};

socket.onclose = (event) => {
    if (event.wasClean) {
        console.debug('Connection closed');
    } else {
        console.debug('Connection dead');
    }
};

socket.onerror = (error) => console.error(error);

document.querySelector('form').onsubmit = (event) => {
    const formData = new FormData(event.target);
    const formProps = Object.fromEntries(formData);
    const message = JSON.stringify(formProps, null, 4);
    socket.send(message);
}