// Verifier for the buffer pilot. Tests transcribe Bun's antichain reps
// where applicable, plus Node-spec edge cases.

use rusty_buffer::*;

// ════════════════════ FACTORIES ════════════════════

#[test]
fn cd_buffer_alloc_zeros_by_default() {
    let b = Buffer::alloc(8);
    assert_eq!(b.len(), 8);
    assert_eq!(b.as_bytes(), &[0u8; 8]);
}

#[test]
fn cd_buffer_alloc_filled_repeats_pattern() {
    let b = Buffer::alloc_filled(7, b"ab");
    // 7 bytes filled with "ab" pattern → "abababa"
    assert_eq!(b.as_bytes(), b"abababa");
}

// CD: `expect(buffer).toBe("message1\nmessage2\nmessage3\n")` — buffer-as-string
#[test]
fn cd_buffer_from_string_roundtrip_utf8() {
    let s = "message1\nmessage2\nmessage3\n";
    let b = Buffer::from_string(s, Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Utf8, 0, None), s);
}

// CD: `expect(buffer?.toString()).toBe("test-value")` — default toString is utf-8
#[test]
fn cd_buffer_to_string_default_utf8() {
    let b = Buffer::from_string("test-value", Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Utf8, 0, None), "test-value");
}

// CD: `expect(buffer).toStrictEqual(Buffer.concat([Buffer.from("bun"), Buffer.alloc(16381)]))`
#[test]
fn cd_buffer_concat_with_alloc() {
    let head = Buffer::from_string("bun", Encoding::Utf8);
    let pad = Buffer::alloc(16381);
    let combined = Buffer::concat(&[head, pad], None);
    assert_eq!(combined.len(), 3 + 16381);
    assert_eq!(&combined.as_bytes()[..3], b"bun");
    assert!(combined.as_bytes()[3..].iter().all(|&b| b == 0));
}

#[test]
fn spec_buffer_concat_total_length_truncates() {
    let a = Buffer::from_string("hello ", Encoding::Utf8);
    let b = Buffer::from_string("world", Encoding::Utf8);
    let c = Buffer::concat(&[a, b], Some(5));
    assert_eq!(c.as_bytes(), b"hello");
}

#[test]
fn spec_buffer_concat_total_length_pads_with_zeros() {
    let a = Buffer::from_string("hi", Encoding::Utf8);
    let c = Buffer::concat(&[a], Some(5));
    assert_eq!(c.as_bytes(), &[b'h', b'i', 0, 0, 0]);
}

#[test]
fn spec_buffer_byte_length_utf8() {
    assert_eq!(Buffer::byte_length("hello", Encoding::Utf8), 5);
    assert_eq!(Buffer::byte_length("héllo", Encoding::Utf8), 6); // é = 2 bytes
}

#[test]
fn spec_buffer_byte_length_utf16le() {
    assert_eq!(Buffer::byte_length("hi", Encoding::Utf16Le), 4);
}

#[test]
fn spec_buffer_is_encoding_known_names() {
    for name in ["utf-8", "utf8", "UTF-8", "latin1", "binary", "ascii", "base64", "hex", "utf-16le"] {
        assert!(Buffer::is_encoding(name), "{} should be a known encoding", name);
    }
}

#[test]
fn spec_buffer_is_encoding_unknown_names() {
    assert!(!Buffer::is_encoding("not-real"));
    assert!(!Buffer::is_encoding(""));
}

// ════════════════════ ENCODINGS ════════════════════

#[test]
fn spec_encoding_utf8_unicode_roundtrip() {
    let s = "héllo, мир! 🌍";
    let b = Buffer::from_string(s, Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Utf8, 0, None), s);
}

#[test]
fn spec_encoding_utf16le_roundtrip() {
    let s = "hi";
    let b = Buffer::from_string(s, Encoding::Utf16Le);
    assert_eq!(b.as_bytes(), &[b'h', 0, b'i', 0]);
    assert_eq!(b.to_string(Encoding::Utf16Le, 0, None), "hi");
}

#[test]
fn spec_encoding_latin1_one_byte_per_char() {
    let s = "héllo"; // é = U+00E9 = byte 0xE9 in latin1
    let b = Buffer::from_string(s, Encoding::Latin1);
    assert_eq!(b.len(), 5);
    assert_eq!(b.as_bytes()[1], 0xE9);
}

#[test]
fn spec_encoding_ascii_strips_high_bit() {
    // ASCII encoding masks the high bit. 'é' (U+00E9 = 0xE9) becomes
    // 0x69 = 'i'. Pure ASCII chars pass through unchanged.
    let b = Buffer::from_string("café", Encoding::Ascii);
    assert_eq!(b.len(), 4);
    assert_eq!(b.as_bytes()[0], b'c');
    assert_eq!(b.as_bytes()[1], b'a');
    assert_eq!(b.as_bytes()[2], b'f');
    assert_eq!(b.as_bytes()[3], 0x69); // 'é' → 0xE9 & 0x7F = 0x69
}

#[test]
fn spec_encoding_base64_encode_basic() {
    let b = Buffer::from_string("hello", Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Base64, 0, None), "aGVsbG8=");
}

#[test]
fn spec_encoding_base64_decode_basic() {
    let b = Buffer::from_string("aGVsbG8=", Encoding::Base64);
    assert_eq!(b.to_string(Encoding::Utf8, 0, None), "hello");
}

#[test]
fn spec_encoding_base64_roundtrip() {
    let original = b"\x00\x01\x02\x03\xFE\xFF";
    let b = Buffer::from_bytes(original);
    let encoded = b.to_string(Encoding::Base64, 0, None);
    let decoded = Buffer::from_string(&encoded, Encoding::Base64);
    assert_eq!(decoded.as_bytes(), original);
}

#[test]
fn spec_encoding_hex_encode_lowercase() {
    let b = Buffer::from_bytes(&[0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(b.to_string(Encoding::Hex, 0, None), "deadbeef");
}

#[test]
fn spec_encoding_hex_decode_case_insensitive() {
    let b1 = Buffer::from_string("DEADBEEF", Encoding::Hex);
    let b2 = Buffer::from_string("deadbeef", Encoding::Hex);
    assert_eq!(b1, b2);
    assert_eq!(b1.as_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
}

// ════════════════════ COMPARE / EQUALS ════════════════════

#[test]
fn spec_buffer_equals_byte_match() {
    let a = Buffer::from_string("hello", Encoding::Utf8);
    let b = Buffer::from_string("hello", Encoding::Utf8);
    assert!(a.equals(&b));
}

#[test]
fn spec_buffer_equals_unequal() {
    let a = Buffer::from_string("hello", Encoding::Utf8);
    let b = Buffer::from_string("world", Encoding::Utf8);
    assert!(!a.equals(&b));
}

#[test]
fn spec_buffer_compare_static() {
    let a = Buffer::from_string("apple", Encoding::Utf8);
    let b = Buffer::from_string("banana", Encoding::Utf8);
    let c = Buffer::from_string("apple", Encoding::Utf8);
    assert_eq!(Buffer::compare_bufs(&a, &b), -1);
    assert_eq!(Buffer::compare_bufs(&b, &a), 1);
    assert_eq!(Buffer::compare_bufs(&a, &c), 0);
}

#[test]
fn spec_buffer_compare_with_ranges() {
    let a = Buffer::from_string("xxhelloxx", Encoding::Utf8);
    let b = Buffer::from_string("--hello--", Encoding::Utf8);
    // Compare a[2..7] against b[2..7] → both "hello" → 0
    assert_eq!(a.compare(&b, 2, Some(7), 2, Some(7)), 0);
}

// ════════════════════ SLICE / SUBARRAY ════════════════════

#[test]
fn spec_buffer_subarray_extracts_range() {
    let b = Buffer::from_string("hello world", Encoding::Utf8);
    let s = b.subarray(6, None);
    assert_eq!(s.to_string(Encoding::Utf8, 0, None), "world");
}

#[test]
fn spec_buffer_subarray_clamps_out_of_range() {
    let b = Buffer::from_string("hi", Encoding::Utf8);
    let s = b.subarray(0, Some(1000));
    assert_eq!(s.as_bytes(), b"hi");
}

#[test]
fn spec_buffer_slice_alias_of_subarray() {
    let b = Buffer::from_string("test", Encoding::Utf8);
    let s1 = b.slice(1, Some(3));
    let s2 = b.subarray(1, Some(3));
    assert_eq!(s1, s2);
    assert_eq!(s1.as_bytes(), b"es");
}

// ════════════════════ INDEX_OF / INCLUDES ════════════════════

#[test]
fn spec_buffer_index_of_finds_substring() {
    let b = Buffer::from_string("hello world hello", Encoding::Utf8);
    assert_eq!(b.index_of_bytes(b"hello", 0), 0);
    assert_eq!(b.index_of_bytes(b"hello", 1), 12);
    assert_eq!(b.index_of_bytes(b"world", 0), 6);
    assert_eq!(b.index_of_bytes(b"missing", 0), -1);
}

#[test]
fn spec_buffer_last_index_of() {
    let b = Buffer::from_string("hello world hello", Encoding::Utf8);
    assert_eq!(b.last_index_of_bytes(b"hello"), 12);
    assert_eq!(b.last_index_of_bytes(b"missing"), -1);
}

#[test]
fn spec_buffer_includes() {
    let b = Buffer::from_string("hello world", Encoding::Utf8);
    assert!(b.includes_bytes(b"world"));
    assert!(!b.includes_bytes(b"missing"));
}

// ════════════════════ FILL / WRITE / COPY ════════════════════

#[test]
fn spec_buffer_fill_byte() {
    let mut b = Buffer::alloc(5);
    b.fill_byte(0xFF, 0, None);
    assert_eq!(b.as_bytes(), &[0xFF; 5]);
}

#[test]
fn spec_buffer_fill_range_only() {
    let mut b = Buffer::alloc(8);
    b.fill_byte(0xAA, 2, Some(5));
    assert_eq!(b.as_bytes(), &[0, 0, 0xAA, 0xAA, 0xAA, 0, 0, 0]);
}

#[test]
fn spec_buffer_fill_with_pattern() {
    let mut b = Buffer::alloc(7);
    b.fill_bytes(b"ab", 0, None);
    assert_eq!(b.as_bytes(), b"abababa");
}

#[test]
fn spec_buffer_write_into_existing() {
    let mut b = Buffer::alloc(10);
    let n = b.write("hello", 0, None, Encoding::Utf8);
    assert_eq!(n, 5);
    assert_eq!(&b.as_bytes()[..5], b"hello");
    assert_eq!(b.as_bytes()[5], 0);
}

#[test]
fn spec_buffer_write_with_offset() {
    let mut b = Buffer::alloc(10);
    let n = b.write("xyz", 3, None, Encoding::Utf8);
    assert_eq!(n, 3);
    assert_eq!(&b.as_bytes()[3..6], b"xyz");
}

#[test]
fn spec_buffer_write_truncates_at_buffer_end() {
    let mut b = Buffer::alloc(5);
    let n = b.write("hello world", 0, None, Encoding::Utf8);
    assert_eq!(n, 5);
    assert_eq!(b.as_bytes(), b"hello");
}

#[test]
fn spec_buffer_copy_basic() {
    let src = Buffer::from_string("hello", Encoding::Utf8);
    let mut dst = Buffer::alloc(10);
    let n = src.copy(&mut dst, 0, 0, None);
    assert_eq!(n, 5);
    assert_eq!(&dst.as_bytes()[..5], b"hello");
}

#[test]
fn spec_buffer_copy_with_target_offset() {
    let src = Buffer::from_string("hello", Encoding::Utf8);
    let mut dst = Buffer::alloc(10);
    let n = src.copy(&mut dst, 3, 0, None);
    assert_eq!(n, 5);
    assert_eq!(&dst.as_bytes()[3..8], b"hello");
}

#[test]
fn spec_buffer_copy_source_range() {
    let src = Buffer::from_string("hello world", Encoding::Utf8);
    let mut dst = Buffer::alloc(5);
    let n = src.copy(&mut dst, 0, 6, None);
    assert_eq!(n, 5);
    assert_eq!(dst.as_bytes(), b"world");
}

// ════════════════════ TO_STRING WITH RANGE ════════════════════

#[test]
fn spec_to_string_partial_range() {
    let b = Buffer::from_string("hello world", Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Utf8, 0, Some(5)), "hello");
    assert_eq!(b.to_string(Encoding::Utf8, 6, None), "world");
}

#[test]
fn spec_to_string_clamps_end_to_length() {
    let b = Buffer::from_string("abc", Encoding::Utf8);
    assert_eq!(b.to_string(Encoding::Utf8, 0, Some(1000)), "abc");
}

// ════════════════════ EDGE CASES ════════════════════

#[test]
fn spec_buffer_alloc_zero_size() {
    let b = Buffer::alloc(0);
    assert_eq!(b.len(), 0);
    assert!(b.is_empty());
}

#[test]
fn spec_buffer_concat_empty_list() {
    let c = Buffer::concat(&[], None);
    assert_eq!(c.len(), 0);
}

#[test]
fn spec_buffer_index_of_empty_needle_returns_offset() {
    let b = Buffer::from_string("hello", Encoding::Utf8);
    // Node behavior: empty needle returns the byteOffset itself
    assert_eq!(b.index_of_bytes(b"", 0), 0);
    assert_eq!(b.index_of_bytes(b"", 3), 3);
}
