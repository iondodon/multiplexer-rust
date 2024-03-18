use std::{env, io::Read, sync::Arc};

use ssh2::Session;
use tokio::net::{TcpListener, TcpStream};

const SCRIPT: &str = r#"
    while true; do
        echo -n "Message"
    done
"#;

struct Node {
    msg: String,
    next: Option<Arc<Node>>,
}

impl Node {
    fn default() -> Node {
        Node {
            msg: String::from_utf8("Message".into()).unwrap(),
            next: None,
        }
    }
}

async fn handle_tcp_client(mut stream: TcpStream) {
    loop {}
}

async fn start_tcp_server(HEAD: Option<Arc<Node>>) {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on port 8080");

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(async move { handle_tcp_client(socket).await });
            }
            Err(e) => {
                println!("Failed to accept peer connection {:?}", e);
                continue;
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let HEAD = Some(Arc::new(Node::default()));

    let HEAD_clone = HEAD.clone();
    tokio::spawn(async { start_tcp_server(HEAD_clone) });

    let username = env::var("USERNAME").expect("USERNAME not set");
    let password = env::var("PASSWORD").expect("PASSWORD not set");
    let tcp = TcpStream::connect("localhost:22").await.unwrap();
    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();
    session.userauth_password(&username, &password).unwrap();
    assert!(session.authenticated());

    let mut channel = session.channel_session().unwrap();
    channel.exec(SCRIPT).unwrap();

    let mut bytes: [u8; 7] = [0; 7];
    loop {
        match channel.read_exact(&mut bytes) {
            Ok(_) => {
                if let Some(ref arc_head) = HEAD {
                    let node = Node {
                        msg: String::from_utf8(bytes.to_vec()).unwrap(),
                        next: None,
                    };
                    arc_head.next = Some(Arc::new(node));
                } else {
                    println!("None head");
                }
            }
            Err(e) => {
                eprintln!("Error reading command output: {:?}", e);
                break;
            }
        }
    }

    channel.wait_close().ok();
    println!(
        "Command finished with exit status: {:?}",
        channel.exit_status().unwrap()
    );
}
