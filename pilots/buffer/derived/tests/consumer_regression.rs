// Consumer-regression suite for buffer.

use rusty_buffer::*;

// ────────── Node fs.readFile — Buffer.from / toString roundtrip ─────
//
// Source: https://github.com/nodejs/node/blob/main/lib/fs.js
//   `fs.readFile(path)` returns a Buffer; `.toString("utf-8")` is the
//   universal decode pattern. Consumers expect lossless utf-8 roundtrip.

#[test]
fn consumer_node_fs_readfile_utf8_roundtrip() {
    let original = "line1\nline2\nline3\n";
    let buf = Buffer::from_string(original, Encoding::Utf8);
    assert_eq!(buf.to_string(Encoding::Utf8, 0, None), original);
}

// ────────── Node http — Buffer.concat for body assembly ─────
//
// Source: https://github.com/nodejs/node/blob/main/lib/_http_incoming.js
//   IncomingMessage assembles body chunks via `Buffer.concat(chunks)`.
//   consumer expectation: concat preserves byte order across chunks.

#[test]
fn consumer_node_http_body_concat_preserves_order() {
    let chunks = vec![
        Buffer::from_string("HTTP/1.1 ", Encoding::Utf8),
        Buffer::from_string("200 ", Encoding::Utf8),
        Buffer::from_string("OK\r\n", Encoding::Utf8),
    ];
    let combined = Buffer::concat(&chunks, None);
    assert_eq!(
        combined.to_string(Encoding::Utf8, 0, None),
        "HTTP/1.1 200 OK\r\n"
    );
}

// ────────── crypto — Buffer.compare for constant-time intent ─────
//
// Source: https://github.com/nodejs/node/blob/main/lib/internal/crypto/...
//   crypto.timingSafeEqual is a separate fn but consumers also use
//   Buffer.compare === 0 for equality checks. consumer expectation:
//   compare returns -1/0/1 deterministically.

#[test]
fn consumer_crypto_compare_deterministic_ordering() {
    let a = Buffer::from_bytes(&[0x00, 0xFF]);
    let b = Buffer::from_bytes(&[0x01, 0x00]);
    assert_eq!(Buffer::compare_bufs(&a, &b), -1);
    assert_eq!(Buffer::compare_bufs(&b, &a), 1);
    let c = Buffer::from_bytes(&[0x00, 0xFF]);
    assert_eq!(Buffer::compare_bufs(&a, &c), 0);
}

// ────────── express body-parser — Buffer.byteLength for Content-Length ─
//
// Source: https://github.com/expressjs/body-parser/blob/master/lib/types/json.js
//   sets Content-Length from Buffer.byteLength(body, "utf-8"). consumer
//   expectation: byteLength matches actual encoded byte count.

#[test]
fn consumer_express_byte_length_matches_encoded_size() {
    let body = r#"{"key":"héllo"}"#; // é is 2 bytes utf-8
    let claimed = Buffer::byte_length(body, Encoding::Utf8);
    let actual = Buffer::from_string(body, Encoding::Utf8).len();
    assert_eq!(claimed, actual);
}

// ────────── jose / oauth-jwt — base64 decode ─────
//
// Source: https://github.com/panva/jose
//   JOSE libraries decode base64 JWT segments to Buffers; consumer
//   expects clean roundtrip on arbitrary byte payloads.

#[test]
fn consumer_jose_base64_arbitrary_bytes_roundtrip() {
    let original = vec![0u8, 1, 2, 254, 255, 128, 64, 32, 16, 8, 4, 2, 1];
    let b = Buffer::from_bytes(&original);
    let encoded = b.to_string(Encoding::Base64, 0, None);
    let decoded = Buffer::from_string(&encoded, Encoding::Base64);
    assert_eq!(decoded.as_bytes(), &original[..]);
}

// ────────── pino logger — Buffer.alloc for fast log writes ─────
//
// Source: https://github.com/pinojs/pino/blob/master/lib/tools.js
//   pre-allocates a Buffer and writes log-line UTF-8 bytes into it via
//   buf.write(line, offset). consumer expectation: write returns bytes
//   written and stops at buffer end.

#[test]
fn consumer_pino_buffer_write_stops_at_end() {
    let mut b = Buffer::alloc(16);
    let n = b.write("this is too long for the buffer", 0, None, Encoding::Utf8);
    assert_eq!(n, 16);
    assert_eq!(&b.as_bytes()[..16], b"this is too long");
}

// ────────── ws (WebSocket library) — frame masking via fill ─────
//
// Source: https://github.com/websockets/ws/blob/master/lib/buffer-util.js
//   masks a buffer in place via XOR; consumer relies on fill to set up
//   mask patterns and on equals for protocol-frame detection.

#[test]
fn consumer_ws_buffer_fill_pattern_then_equals() {
    let mut b = Buffer::alloc(8);
    b.fill_bytes(&[0xCA, 0xFE], 0, None);
    let expected = Buffer::from_bytes(&[0xCA, 0xFE, 0xCA, 0xFE, 0xCA, 0xFE, 0xCA, 0xFE]);
    assert!(b.equals(&expected));
}

// ────────── busboy — Buffer.indexOf for boundary scanning ─────
//
// Source: https://github.com/mscdex/busboy/blob/master/lib/types/multipart.js
//   scans incoming chunks for boundary delimiters via indexOf. consumer
//   expects -1 on no match, byte index on match.

#[test]
fn consumer_busboy_index_of_boundary_scanning() {
    let body = Buffer::from_string(
        "preamble\r\n--boundary123\r\nheaders\r\n\r\nbody\r\n--boundary123--",
        Encoding::Utf8,
    );
    let idx = body.index_of_bytes(b"--boundary123", 0);
    assert!(idx > 0);
    assert_eq!(idx, 10);
    let next = body.index_of_bytes(b"--boundary123", (idx + 1) as usize);
    assert!(next > idx);
}

// ────────── postgres driver — hex encoding for binary fields ─────
//
// Source: https://github.com/brianc/node-postgres/blob/master/packages/pg/lib/types/textParsers.js
//   pg's bytea text format is `\xDEADBEEF`; the parser strips the prefix
//   and Buffer.from(hex_string, "hex") restores bytes.

#[test]
fn consumer_postgres_hex_decode_bytea() {
    let bytea_text = "DEADBEEF";
    let b = Buffer::from_string(bytea_text, Encoding::Hex);
    assert_eq!(b.as_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
}

// ────────── multer — utf-16le for filename surrogate pairs ─────
//
// Source: https://github.com/expressjs/multer/blob/master/lib/file-appender.js
//   filenames may arrive utf-16le from older clients; multer decodes via
//   Buffer.toString("utf-16le").

#[test]
fn consumer_multer_utf16le_filename_decode() {
    let bytes = [b'h', 0, b'i', 0, b'!', 0];
    let b = Buffer::from_bytes(&bytes);
    assert_eq!(b.to_string(Encoding::Utf16Le, 0, None), "hi!");
}

// ────────── Bun-specific — Buffer.alloc(16381) padding pattern ─────
//
// Source: Bun test fs.test.ts:3260 cited in CD; large pre-allocated
// padding for read buffers.

#[test]
fn consumer_bun_alloc_padding_then_concat() {
    let head = Buffer::from_string("bun", Encoding::Utf8);
    let pad = Buffer::alloc(16381);
    let combined = Buffer::concat(&[head.clone(), pad], None);
    let expected_head = combined.subarray(0, Some(3));
    assert!(expected_head.equals(&head));
    let expected_pad_first = combined.as_bytes()[3];
    assert_eq!(expected_pad_first, 0);
}
