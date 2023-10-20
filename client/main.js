const socket = new WebSocket("wss://javascript.info/article/websocket/chat/ws", ["soap", "wamp"]);

socket.onopen = (_) => {
    console.debug("Connection established");    
};

socket.onmessage = (event) => {
    const messageListItemEl = document.createElement('li');
    messageListItemEl.textContent = event.data;
    document.getElementById('messages').prepend(messageListItemEl);
};

socket.onclose = (event) => {
    if (event.wasClean) {
        console.debug('Connection closed');
    } else {
        console.debug('Connection dead');
    }
};

socket.onerror = (error) => console.error(error);

const formRef = document.querySelector('form');
formRef.onsubmit = (event) => {
    const formData = new FormData(event.target);
    const formProps = Object.fromEntries(formData);
    socket.send(formProps['message']);
}