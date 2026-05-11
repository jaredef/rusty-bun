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

// ─────────────── Async listener (option A: thread + mpsc queue) ────

#[test]
fn async_listener_bind_returns_id_and_addr() {
    let (id, addr) = listener_bind_async("127.0.0.1:0").unwrap();
    assert!(addr.starts_with("127.0.0.1:"));
    assert_eq!(handle_kind(id).unwrap(), "async-listener");
    assert_eq!(async_listener_addr(id).unwrap(), addr);
    listener_stop_async(id).unwrap();
}

#[test]
fn async_listener_poll_timeout_returns_none() {
    let (id, _addr) = listener_bind_async("127.0.0.1:0").unwrap();
    // No client connects; poll should time out and return None.
    let r = listener_poll(id, 50).unwrap();
    assert!(r.is_none(), "expected None on timeout, got {:?}", r);
    listener_stop_async(id).unwrap();
}

#[test]
fn async_listener_delivers_connection_event() {
    let (id, addr) = listener_bind_async("127.0.0.1:0").unwrap();

    // Client thread connects.
    let server_addr = addr.clone();
    let client = thread::spawn(move || {
        let cid = stream_connect(&server_addr).unwrap();
        stream_write_all(cid, b"hello").unwrap();
        // Wait for server to read + echo back.
        let echo = stream_read(cid, 1024).unwrap();
        assert_eq!(&echo, b"hello");
        handle_close(cid).unwrap();
    });

    // Main thread polls for a connection event.
    let ev = listener_poll(id, 1000).unwrap();
    let (stream_id, peer) = match ev {
        Some(AsyncEvent::Connection { stream_id, peer }) => (stream_id, peer),
        other => panic!("expected Connection event, got {:?}", other),
    };
    assert!(peer.starts_with("127.0.0.1:"));

    // Read what the client sent, echo back.
    let bytes = stream_read(stream_id, 1024).unwrap();
    assert_eq!(&bytes, b"hello");
    stream_write_all(stream_id, &bytes).unwrap();
    handle_close(stream_id).unwrap();

    client.join().unwrap();
    listener_stop_async(id).unwrap();
}

#[test]
fn async_listener_multiple_connections() {
    let (id, addr) = listener_bind_async("127.0.0.1:0").unwrap();
    let server_addr = addr.clone();

    // Spawn three client connections.
    let mut clients = vec![];
    for i in 0..3 {
        let a = server_addr.clone();
        clients.push(thread::spawn(move || {
            let cid = stream_connect(&a).unwrap();
            stream_write_all(cid, format!("msg-{}", i).as_bytes()).unwrap();
            let echo = stream_read(cid, 1024).unwrap();
            assert_eq!(&echo, format!("ack-{}", i).as_bytes());
            handle_close(cid).unwrap();
        }));
    }

    // Main thread accepts all three.
    for _ in 0..3 {
        let ev = listener_poll(id, 2000).unwrap();
        let sid = match ev {
            Some(AsyncEvent::Connection { stream_id, .. }) => stream_id,
            other => panic!("expected Connection, got {:?}", other),
        };
        let bytes = stream_read(sid, 1024).unwrap();
        // bytes is "msg-N"; respond with "ack-N".
        let n_char = bytes[4]; // "msg-N" → byte at index 4 is N's ASCII
        let resp = format!("ack-{}", (n_char - b'0'));
        stream_write_all(sid, resp.as_bytes()).unwrap();
        handle_close(sid).unwrap();
    }

    for c in clients { c.join().unwrap(); }
    listener_stop_async(id).unwrap();
}

#[test]
fn async_listener_stop_unblocks_subsequent_poll() {
    let (id, _addr) = listener_bind_async("127.0.0.1:0").unwrap();
    listener_stop_async(id).unwrap();
    // After stop, the handle is gone; poll returns NotFound.
    assert!(matches!(listener_poll(id, 100), Err(SocketError::NotFound)));
}

#[test]
fn async_listener_kind_distinguishes_from_sync() {
    let (sync_id, _) = listener_bind("127.0.0.1:0").unwrap();
    let (async_id, _) = listener_bind_async("127.0.0.1:0").unwrap();
    assert_eq!(handle_kind(sync_id).unwrap(), "listener");
    assert_eq!(handle_kind(async_id).unwrap(), "async-listener");
    // Async ops on sync handle → WrongKind.
    assert!(matches!(listener_poll(sync_id, 10), Err(SocketError::WrongKind)));
    // Sync accept on async handle → WrongKind.
    assert!(matches!(listener_accept(async_id), Err(SocketError::WrongKind)));
    handle_close(sync_id).unwrap();
    listener_stop_async(async_id).unwrap();
}
