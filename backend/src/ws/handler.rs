use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_connection)
}

async fn handle_connection(mut socket: WebSocket) {
    // 这里可以实现 WebSocket 连接的处理逻辑
    // 例如：分数变化推送、通知推送等
    while let Some(Ok(message)) = socket.recv().await {
        // 处理接收到的消息
        if let axum::extract::ws::Message::Text(text) = message {
            // 这里可以处理客户端发送的消息
            println!("Received message: {}", text);
        }
    }
}
