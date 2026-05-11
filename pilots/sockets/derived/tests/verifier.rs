// sockets verifier — in-process TCP loopback round-trips.

use rusty_sockets::*;
use std::thread;
use std::time::Duration;

#[test]
fn bind_loopback_any_port() {
    let (id, addr) = listener_bind("127.0.0.1:0").unwrap();
    assert!(addr.starts_with("127.0.0.1:"));
    assert_eq!(handle_kind(id).unwrap(), "listener");
    handle_close(id).unwrap();
}

#[test]
fn loopback_echo_roundtrip() {
    let (lid, addr) = listener_bind("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    // Server thread: accept + echo back what's received.
    let server = thread::spawn(move || {
        let (sid, _peer) = listener_accept(lid).unwrap();
        let bytes = stream_read(sid, 1024).unwrap();
        stream_write_all(sid, &bytes).unwrap();
        handle_close(sid).unwrap();
        handle_close(lid).unwrap();
    });

    // Client: connect, send, read echo.
    let cid = stream_connect(&server_addr).unwrap();
    stream_write_all(cid, b"ping").unwrap();
    let echoed = stream_read(cid, 1024).unwrap();
    assert_eq!(&echoed, b"ping");
    handle_close(cid).unwrap();

    server.join().unwrap();
}

#[test]
fn http_like_request_response() {
    let (lid, addr) = listener_bind("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    let server = thread::spawn(move || {
        let (sid, _) = listener_accept(lid).unwrap();
        // Read request bytes (in real use, parse them via http-codec).
        let req = stream_read(sid, 4096).unwrap();
        // Verify we got the request bytes.
        assert!(req.starts_with(b"GET /health HTTP/1.1"));
        // Send a response.
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nhello world";
        stream_write_all(sid, response).unwrap();
        handle_close(sid).unwrap();
        handle_close(lid).unwrap();
    });

    let cid = stream_connect(&server_addr).unwrap();
    let request = b"GET /health HTTP/1.1\r\nHost: example.com\r\n\r\n";
    stream_write_all(cid, request).unwrap();
    let resp = stream_read(cid, 4096).unwrap();
    assert!(resp.starts_with(b"HTTP/1.1 200 OK"));
    assert!(resp.ends_with(b"hello world"));
    handle_close(cid).unwrap();

    server.join().unwrap();
}

#[test]
fn connect_to_unreachable_errors() {
    // 0.0.0.0:1 is not a valid connect target; should fail quickly.
    let r = stream_connect_timeout("127.0.0.1:1", 100);
    assert!(r.is_err());
}

#[test]
fn orderly_close_returns_empty_read() {
    let (lid, addr) = listener_bind("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    let server = thread::spawn(move || {
        let (sid, _) = listener_accept(lid).unwrap();
        // Close immediately without writing.
        handle_close(sid).unwrap();
        handle_close(lid).unwrap();
    });

    let cid = stream_connect(&server_addr).unwrap();
    // Give the server time to close.
    thread::sleep(Duration::from_millis(50));
    let buf = stream_read(cid, 1024).unwrap();
    assert!(buf.is_empty(), "orderly close should return empty read");
    handle_close(cid).unwrap();

    server.join().unwrap();
}

#[test]
fn peer_and_local_addr_set() {
    let (lid, addr) = listener_bind("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    let server = thread::spawn(move || {
        let (sid, peer) = listener_accept(lid).unwrap();
        // Server's peer == client's local
        let server_peer = stream_peer_addr(sid).unwrap();
        let server_local = stream_local_addr(sid).unwrap();
        assert_eq!(server_peer, peer);
        assert!(server_local.starts_with("127.0.0.1:"));
        handle_close(sid).unwrap();
        handle_close(lid).unwrap();
    });

    let cid = stream_connect(&server_addr).unwrap();
    let client_peer = stream_peer_addr(cid).unwrap();
    let client_local = stream_local_addr(cid).unwrap();
    assert_eq!(client_peer, server_addr);
    assert!(client_local.starts_with("127.0.0.1:"));
    handle_close(cid).unwrap();

    server.join().unwrap();
}

#[test]
fn handle_close_invalidates() {
    let (id, _) = listener_bind("127.0.0.1:0").unwrap();
    handle_close(id).unwrap();
    // Subsequent close should fail.
    assert!(matches!(handle_close(id), Err(SocketError::NotFound)));
    // Operations on the id should fail.
    assert!(matches!(listener_accept(id), Err(SocketError::NotFound)));
}

#[test]
fn wrong_kind_errors() {
    let (lid, _) = listener_bind("127.0.0.1:0").unwrap();
    // Trying to use a listener id as a stream → wrong kind.
    assert!(matches!(stream_read(lid, 100), Err(SocketError::WrongKind)));
    assert!(matches!(stream_write(lid, b"x"), Err(SocketError::WrongKind)));
    handle_close(lid).unwrap();
}

#[test]
fn multi_request_per_connection() {
    let (lid, addr) = listener_bind("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    let server = thread::spawn(move || {
        let (sid, _) = listener_accept(lid).unwrap();
        // Two requests / two responses.
        for _ in 0..2 {
            let req = stream_read(sid, 1024).unwrap();
            assert!(!req.is_empty());
            stream_write_all(sid, b"OK\n").unwrap();
        }
        handle_close(sid).unwrap();
        handle_close(lid).unwrap();
    });

    let cid = stream_connect(&server_addr).unwrap();
    stream_write_all(cid, b"req1\n").unwrap();
    let r1 = stream_read(cid, 1024).unwrap();
    assert_eq!(&r1, b"OK\n");
    stream_write_all(cid, b"req2\n").unwrap();
    let r2 = stream_read(cid, 1024).unwrap();
    assert_eq!(&r2, b"OK\n");
    handle_close(cid).unwrap();

    server.join().unwrap();
}
