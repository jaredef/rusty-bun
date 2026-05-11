// Verifier suite for rusty-websocket. Real RFC 6455 + RFC 6455 §1.3
// vectors.

use rusty_websocket::*;

// ─── Sec-WebSocket-Accept derivation (RFC 6455 §1.3 example) ────────

#[test]
fn accept_rfc6455_example() {
    // RFC 6455 §1.3: client key "dGhlIHNhbXBsZSBub25jZQ==" must produce
    // accept "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let expected = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
    assert_eq!(derive_accept(key), expected);
}

#[test]
fn accept_verify() {
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let server = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
    assert!(verify_accept(key, server));
    assert!(!verify_accept(key, "WRONG"));
}

#[test]
fn generate_key_unique_and_length() {
    let k1 = generate_key().unwrap();
    let k2 = generate_key().unwrap();
    assert_ne!(k1, k2);
    // 16-byte input + base64 → 24 chars including 2 padding chars.
    assert_eq!(k1.len(), 24);
    assert!(k1.ends_with("=="));
}

// ─── Frame encoding/decoding ────────────────────────────────────────

#[test]
fn unmasked_text_frame_roundtrip() {
    // RFC 6455 §5.7 example: A single-frame unmasked text message
    // containing "Hello": 0x81 0x05 0x48 0x65 0x6c 0x6c 0x6f.
    let frame = Frame {
        fin: true, opcode: Opcode::Text, payload: b"Hello".to_vec(), mask: None,
    };
    let bytes = encode_frame(&frame).unwrap();
    assert_eq!(bytes, vec![0x81, 0x05, b'H', b'e', b'l', b'l', b'o']);
    let (decoded, n) = decode_frame(&bytes).unwrap();
    assert_eq!(n, bytes.len());
    assert!(decoded.fin);
    assert_eq!(decoded.opcode, Opcode::Text);
    assert_eq!(decoded.payload, b"Hello");
    assert!(decoded.mask.is_none());
}

#[test]
fn masked_text_frame_decodes() {
    // RFC 6455 §5.7 example: masked "Hello" with mask 0x37 0xfa 0x21 0x3d:
    // 0x81 0x85 0x37 0xfa 0x21 0x3d 0x7f 0x9f 0x4d 0x51 0x58.
    let bytes = [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58];
    let (decoded, n) = decode_frame(&bytes).unwrap();
    assert_eq!(n, bytes.len());
    assert!(decoded.fin);
    assert_eq!(decoded.opcode, Opcode::Text);
    assert_eq!(decoded.payload, b"Hello");
    assert_eq!(decoded.mask, Some([0x37, 0xfa, 0x21, 0x3d]));
}

#[test]
fn masked_encode_matches_rfc_example() {
    let frame = Frame {
        fin: true, opcode: Opcode::Text, payload: b"Hello".to_vec(),
        mask: Some([0x37, 0xfa, 0x21, 0x3d]),
    };
    let bytes = encode_frame(&frame).unwrap();
    assert_eq!(bytes, vec![0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58]);
}

#[test]
fn binary_frame_roundtrip() {
    let frame = Frame {
        fin: true, opcode: Opcode::Binary, payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
        mask: None,
    };
    let bytes = encode_frame(&frame).unwrap();
    let (decoded, _) = decode_frame(&bytes).unwrap();
    assert_eq!(decoded.opcode, Opcode::Binary);
    assert_eq!(decoded.payload, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn fragmented_frames() {
    let f1 = Frame { fin: false, opcode: Opcode::Text, payload: b"Hel".to_vec(), mask: None };
    let f2 = Frame { fin: true, opcode: Opcode::Continuation, payload: b"lo".to_vec(), mask: None };
    let mut combined = encode_frame(&f1).unwrap();
    combined.extend(encode_frame(&f2).unwrap());
    let (d1, n1) = decode_frame(&combined).unwrap();
    assert!(!d1.fin);
    assert_eq!(d1.opcode, Opcode::Text);
    assert_eq!(d1.payload, b"Hel");
    let (d2, _) = decode_frame(&combined[n1..]).unwrap();
    assert!(d2.fin);
    assert_eq!(d2.opcode, Opcode::Continuation);
    assert_eq!(d2.payload, b"lo");
}

#[test]
fn medium_length_frame() {
    // Payload length 126..=65535 uses 2-byte extended length.
    let payload = vec![0x42; 200];
    let frame = Frame { fin: true, opcode: Opcode::Binary, payload: payload.clone(), mask: None };
    let bytes = encode_frame(&frame).unwrap();
    assert_eq!(bytes[1], 126);  // length indicator
    assert_eq!(&bytes[2..4], &[0x00, 0xC8]);  // 200 in BE
    let (decoded, _) = decode_frame(&bytes).unwrap();
    assert_eq!(decoded.payload.len(), 200);
    assert_eq!(decoded.payload, payload);
}

#[test]
fn large_length_frame() {
    // Payload length >65535 uses 8-byte extended length.
    let payload = vec![0x11; 70_000];
    let frame = Frame { fin: true, opcode: Opcode::Binary, payload: payload.clone(), mask: None };
    let bytes = encode_frame(&frame).unwrap();
    assert_eq!(bytes[1], 127);
    let (decoded, _) = decode_frame(&bytes).unwrap();
    assert_eq!(decoded.payload.len(), 70_000);
    assert_eq!(decoded.payload[0], 0x11);
    assert_eq!(decoded.payload[69_999], 0x11);
}

#[test]
fn ping_pong_roundtrip() {
    let ping = Frame { fin: true, opcode: Opcode::Ping, payload: b"ping".to_vec(), mask: None };
    let pong = Frame { fin: true, opcode: Opcode::Pong, payload: b"pong".to_vec(), mask: None };
    let (d1, _) = decode_frame(&encode_frame(&ping).unwrap()).unwrap();
    let (d2, _) = decode_frame(&encode_frame(&pong).unwrap()).unwrap();
    assert_eq!(d1.opcode, Opcode::Ping);
    assert_eq!(d2.opcode, Opcode::Pong);
}

#[test]
fn close_frame_with_code_and_reason() {
    let payload = encode_close(Some(1000), "normal closure");
    assert_eq!(&payload[..2], &[0x03, 0xE8]);  // 1000 in BE
    assert_eq!(&payload[2..], b"normal closure");
    let parsed = decode_close(&payload);
    assert_eq!(parsed.code, Some(1000));
    assert_eq!(parsed.reason, "normal closure");
}

#[test]
fn close_frame_empty_payload() {
    let parsed = decode_close(&[]);
    assert!(parsed.code.is_none());
    assert_eq!(parsed.reason, "");
}

#[test]
fn control_frame_too_long_rejected() {
    let frame = Frame { fin: true, opcode: Opcode::Ping, payload: vec![0; 200], mask: None };
    assert!(matches!(encode_frame(&frame), Err(WsError::ControlTooLong)));
}

#[test]
fn fragmented_control_frame_rejected_on_decode() {
    // FIN=0 + opcode=ping (0x9) is invalid per RFC §5.4.
    let buf = [0x09, 0x00];  // ping, FIN=0, length 0
    assert!(matches!(decode_frame(&buf), Err(WsError::ControlFragmented)));
}

#[test]
fn reserved_bits_rejected() {
    // RSV1=1 (0x40) with opcode=text and no extension negotiated.
    let buf = [0xC1, 0x00];  // FIN=1, RSV1=1, opcode=text, len=0
    assert!(matches!(decode_frame(&buf), Err(WsError::ReservedBitsSet)));
}

#[test]
fn truncated_frame_rejected() {
    // Header claims length 10, only 3 bytes follow.
    let buf = [0x82, 0x0A, 0x01, 0x02, 0x03];
    assert!(matches!(decode_frame(&buf), Err(WsError::UnexpectedEnd)));
}

#[test]
fn invalid_opcode_rejected() {
    let buf = [0x83, 0x00];  // opcode=3 reserved per RFC 6455 §5.2
    assert!(matches!(decode_frame(&buf), Err(WsError::InvalidOpcode(_))));
}
