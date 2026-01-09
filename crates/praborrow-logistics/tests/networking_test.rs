use praborrow_logistics::{Transport, TokioTransport};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;

#[tokio::test]
async fn test_networking_echo() {
    // 1. Start Echo Server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = task::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 1024];
        loop {
            match socket.read(&mut buf).await {
                Ok(0) => return, // EOF
                Ok(n) => {
                    if socket.write_all(&buf[0..n]).await.is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });

    // 2. Client Connect
    let mut transport = TokioTransport::connect(addr).await.expect("Failed to connect");
    
    // 3. Send
    let msg = b"Hello PraBorrow";
    transport.send(msg).await.expect("Send failed");
    
    // 4. Recv
    let mut buf = [0u8; 1024];
    let n = transport.recv(&mut buf).await.expect("Recv failed");
    
    assert_eq!(&buf[0..n], msg);
}
