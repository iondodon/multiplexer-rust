use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    process::Command as TokioCommand,
    sync::broadcast,
};

const SCRIPT: &str = r#"
while true; do
    echo "Message"
done
"#;

#[derive(Clone)]
struct Message {
    content: String,
    sequence: u64,
}

async fn message_listener(tx: broadcast::Sender<Message>) {
    let mut sequence = 0u64;
    let mut child = TokioCommand::new("bash")
        .arg("-c")
        .arg(SCRIPT)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start script");

    let mut stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 1];

    loop {
        match stdout.read_exact(&mut temp_buf).await {
            Ok(_) => {
                if temp_buf[0] == b'\n' {
                    if let Ok(content) = String::from_utf8(buffer.clone()) {
                        sequence += 1;
                        let message = Message { content, sequence };
                        println!(
                            "Received from script: {} (seq: {})",
                            message.content, message.sequence
                        );
                        if tx.send(message).is_err() {
                            println!("All clients disconnected, stopping message listener");
                            break;
                        }
                    }
                    buffer.clear();
                } else {
                    buffer.push(temp_buf[0]);
                }
            }
            Err(e) => {
                eprintln!("Error reading from script: {}", e);
                break;
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream, mut rx: broadcast::Receiver<Message>) {
    println!("New client connected: {}", stream.peer_addr().unwrap());

    while let Ok(msg) = rx.recv().await {
        if let Err(e) = stream.write_all(msg.content.as_bytes()).await {
            eprintln!("Failed to write to client: {}", e);
            break;
        }
        if let Err(e) = stream.write_all(b"\n").await {
            eprintln!("Failed to write newline to client: {}", e);
            break;
        }
        println!(
            "Sent message {} to client {}",
            msg.sequence,
            stream.peer_addr().unwrap()
        );
    }
    println!("Client disconnected: {}", stream.peer_addr().unwrap());
}

async fn accept_connections(listener: TcpListener, tx: broadcast::Sender<Message>) {
    println!("Server listening on port 8080");

    while let Ok((socket, _)) = listener.accept().await {
        let rx = tx.subscribe();
        tokio::spawn(async move {
            handle_client(socket, rx).await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a broadcast channel for message distribution
    let (tx, _) = broadcast::channel(1024);

    // Start the message listener
    let message_tx = tx.clone();
    let message_handle = tokio::spawn(async move {
        message_listener(message_tx).await;
    });

    // Start accepting client connections
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let server_handle = tokio::spawn(async move {
        accept_connections(listener, tx).await;
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    // Shutdown gracefully
    message_handle.abort();
    server_handle.abort();

    Ok(())
}
