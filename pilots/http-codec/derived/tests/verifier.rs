// http-codec verifier — RFC 7230 wire-format vectors + Bun-emitted bytes.

use rusty_http_codec::*;

// Helper: build a request byte buffer from a string with explicit CRLFs.
fn b(s: &str) -> Vec<u8> { s.as_bytes().to_vec() }

#[test]
fn parse_simple_get() {
    let bytes = b("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
    let r = parse_request(&bytes).unwrap();
    assert_eq!(r.method, "GET");
    assert_eq!(r.target, "/");
    assert_eq!(r.version, "HTTP/1.1");
    assert_eq!(r.headers, vec![("Host".to_string(), "example.com".to_string())]);
    assert!(r.body.is_empty());
}

#[test]
fn parse_post_with_content_length() {
    let bytes = b("POST /api HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{\"name\":\"x\"}\n");
    let r = parse_request(&bytes).unwrap();
    assert_eq!(r.method, "POST");
    assert_eq!(r.target, "/api");
    assert_eq!(r.body.len(), 13);
    assert_eq!(&r.body[..], b"{\"name\":\"x\"}\n");
}

#[test]
fn parse_case_insensitive_headers() {
    let bytes = b("GET / HTTP/1.1\r\nCONTENT-LENGTH: 0\r\nhost: x\r\n\r\n");
    let r = parse_request(&bytes).unwrap();
    assert_eq!(r.headers.len(), 2);
    // Headers preserved as-cased.
    assert_eq!(r.headers[0].0, "CONTENT-LENGTH");
    assert_eq!(r.headers[1].0, "host");
}

#[test]
fn parse_response_200() {
    let bytes = b("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello");
    let r = parse_response(&bytes).unwrap();
    assert_eq!(r.status, 200);
    assert_eq!(r.reason, "OK");
    assert_eq!(&r.body[..], b"hello");
}

#[test]
fn parse_response_404_with_reason_phrase_spaces() {
    let bytes = b("HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nnot here!");
    let r = parse_response(&bytes).unwrap();
    assert_eq!(r.status, 404);
    assert_eq!(r.reason, "Not Found");
}

#[test]
fn parse_truncated_is_error() {
    // No header-end CRLF-CRLF.
    let bytes = b("GET / HTTP/1.1\r\nHost: x");
    assert_eq!(parse_request(&bytes), Err(CodecError::Truncated));
}

#[test]
fn parse_bad_version_is_error() {
    let bytes = b("GET / FTP/3.0\r\n\r\n");
    assert!(matches!(parse_request(&bytes), Err(CodecError::BadVersion(_))));
}

#[test]
fn parse_invalid_status_is_error() {
    let bytes = b("HTTP/1.1 NOT_A_NUMBER OK\r\n\r\n");
    assert!(matches!(parse_response(&bytes), Err(CodecError::BadStatus(_))));
}

#[test]
fn serialize_simple_response() {
    let headers = vec![("Content-Type".to_string(), "text/plain".to_string())];
    let body = b"hello";
    let out = serialize_response(200, "OK", &headers, body);
    // Auto-injected Content-Length.
    assert_eq!(out, b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello");
}

#[test]
fn serialize_request_with_existing_content_length() {
    let headers = vec![
        ("Host".to_string(), "x".to_string()),
        ("Content-Length".to_string(), "0".to_string()),
    ];
    let out = serialize_request("DELETE", "/r/1", &headers, b"");
    assert_eq!(out, b"DELETE /r/1 HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n\r\n");
}

#[test]
fn roundtrip_request() {
    let original = b("POST /api HTTP/1.1\r\nHost: example.com\r\nContent-Length: 11\r\n\r\nhello world");
    let parsed = parse_request(&original).unwrap();
    let reserialized = serialize_request(&parsed.method, &parsed.target, &parsed.headers, &parsed.body);
    // Reserialized bytes have headers in same order; should match original.
    assert_eq!(reserialized, original);
}

#[test]
fn chunked_encode_basic() {
    let out = chunked_encode(&[b"hello ", b"world"]);
    assert_eq!(out, b"6\r\nhello \r\n5\r\nworld\r\n0\r\n\r\n");
}

#[test]
fn chunked_decode_basic() {
    let bytes = b"6\r\nhello \r\n5\r\nworld\r\n0\r\n\r\n";
    let out = chunked_decode(bytes).unwrap();
    assert_eq!(out, b"hello world");
}

#[test]
fn chunked_decode_with_extension_ignored() {
    // chunk-ext is allowed after the size; we strip it.
    let bytes = b"6;name=value\r\nhello \r\n5\r\nworld\r\n0\r\n\r\n";
    let out = chunked_decode(bytes).unwrap();
    assert_eq!(out, b"hello world");
}

#[test]
fn chunked_roundtrip_via_parse_response() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");
    bytes.extend_from_slice(&chunked_encode(&[b"chunk1 ", b"chunk2"]));
    let r = parse_response(&bytes).unwrap();
    assert_eq!(&r.body[..], b"chunk1 chunk2");
}

#[test]
fn content_length_overrides_remaining_bytes() {
    // Body in buffer is longer than declared Content-Length: take only the declared length.
    let bytes = b("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhellothereextra");
    let r = parse_response(&bytes).unwrap();
    assert_eq!(&r.body[..], b"hello");
}
