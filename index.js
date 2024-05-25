const ws = new WebSocket("ws://localhost:3000/ws");

ws.onopen = () => {
  console.log("connected");
  ws.send("hello server");
};

ws.onmessage = (msg) => {
  console.log(msg.data);
};
