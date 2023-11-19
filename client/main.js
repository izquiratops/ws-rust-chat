const MAX_HISTORY_SIZE = 60;
const socket = new WebSocket("ws://localhost:3030/chat");

socket.onmessage = handleWebsocketsMessage;
document.querySelector('form').onsubmit = handleSubmitForm;

function handleWebsocketsMessage(event) {
    const newMessage = JSON.parse(event.data);

    // Append new message
    const chatContainerEl = document.getElementById('chat-container');
    console.debug('!!');
    const chatEntryContainerEl = document.createElement('div');
    chatEntryContainerEl.id = 'chat-entry';

    const usernameEl = document.createElement('span');
    usernameEl.innerText = newMessage['username'];
    
    const messageEl = document.createElement('span');
    messageEl.innerText = newMessage['message'];

    chatEntryContainerEl.appendChild(usernameEl);
    chatEntryContainerEl.appendChild(messageEl);
    chatContainerEl.prepend(chatEntryContainerEl);

    // Clear older messages to keep under the maximum number
    const historyMessages = chatContainerEl.getElementsByTagName('div');
    while (historyMessages.length > MAX_HISTORY_SIZE) {
        const messageToBeRemoved = historyMessages[historyMessages.length - 1];
        chatContainerEl.removeChild(messageToBeRemoved);
    }
}

function handleSubmitForm(event) {
    const formData = new FormData(event.target);
    const formProps = Object.fromEntries(formData);
    const message = JSON.stringify(formProps, null, 4);
    socket.send(message);

    // clears the current message input
    document.getElementById('input-message').value = '';
}