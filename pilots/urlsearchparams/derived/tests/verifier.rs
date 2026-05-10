// Verifier for the URLSearchParams pilot. Each test ties to one antichain
// representative from the v0.13b enriched constraint corpus or to a WHATWG
// URL §5.2 spec invariant. Pass = derivation satisfies the constraint.
//
// CD-USP = runs/2026-05-10-deno-v0.13b-spec-batch/constraints/urlsearchparams.constraints.md
// SPEC = https://url.spec.whatwg.org/#interface-urlsearchparams

use rusty_urlsearchparams::*;

// ─────────── CROSS-CORROBORATED reps (highest tier) ──────────
// CD-USP / URLS2 antichain: Deno tests' toString assertions.
// `assertEquals(searchParams, "str=this+string+has+spaces+in+it")`
#[test]
fn cd_cross_tostring_space_to_plus() {
    let mut p = URLSearchParams::new();
    p.append("str", "this string has spaces in it");
    assert_eq!(p.to_string(), "str=this+string+has+spaces+in+it");
}

// `assertEquals(searchParams, "str=hello%2C+world%21")`
#[test]
fn cd_cross_tostring_percent_encoding() {
    let mut p = URLSearchParams::new();
    p.append("str", "hello, world!");
    assert_eq!(p.to_string(), "str=hello%2C+world%21");
}

// ─────────── Constructor + init forms (CD URLS3 / spec) ──────────

#[test]
fn cd_construct_empty() {
    let p = URLSearchParams::new();
    assert_eq!(p.size(), 0);
    assert_eq!(p.to_string(), "");
}

#[test]
fn cd_construct_from_query_with_leading_question() {
    let p = URLSearchParams::from_query("?a=1&b=2");
    assert_eq!(p.size(), 2);
    assert_eq!(p.get("a"), Some("1"));
    assert_eq!(p.get("b"), Some("2"));
}

#[test]
fn cd_construct_from_query_without_leading_question() {
    let p = URLSearchParams::from_query("a=1&b=2");
    assert_eq!(p.size(), 2);
}

#[test]
fn cd_construct_decodes_plus_and_percent() {
    let p = URLSearchParams::from_query("greeting=hello%2C+world%21");
    assert_eq!(p.get("greeting"), Some("hello, world!"));
}

#[test]
fn cd_construct_from_pairs() {
    let p = URLSearchParams::from_pairs(&[("a", "1"), ("b", "2"), ("a", "3")]);
    assert_eq!(p.size(), 3);
    assert_eq!(p.get_all("a"), vec!["1", "3"]);
}

// ─────────── append (CD URLSearchParams.prototype.append) ──────────

#[test]
fn cd_append_never_replaces() {
    let mut p = URLSearchParams::new();
    p.append("k", "1");
    p.append("k", "2");
    assert_eq!(p.get_all("k"), vec!["1", "2"]);
}

// ─────────── delete (CD URLSearchParams.prototype.delete) ──────────

#[test]
fn cd_delete_by_name_removes_all() {
    let mut p = URLSearchParams::from_pairs(&[("a", "1"), ("b", "2"), ("a", "3")]);
    p.delete("a");
    assert_eq!(p.size(), 1);
    assert!(!p.has("a"));
}

#[test]
fn cd_delete_with_value_removes_only_matching() {
    let mut p = URLSearchParams::from_pairs(&[("a", "1"), ("a", "2"), ("a", "3")]);
    p.delete_with_value("a", "2");
    assert_eq!(p.get_all("a"), vec!["1", "3"]);
}

// ─────────── get + getAll (CD URLSearchParams.prototype.{get,getAll}) ──────

#[test]
fn cd_get_returns_first_match() {
    let p = URLSearchParams::from_pairs(&[("k", "first"), ("k", "second")]);
    assert_eq!(p.get("k"), Some("first"));
}

#[test]
fn cd_get_returns_none_when_absent() {
    let p = URLSearchParams::from_pairs(&[("k", "v")]);
    assert_eq!(p.get("missing"), None);
}

#[test]
fn cd_getall_returns_all_matches() {
    let p = URLSearchParams::from_pairs(&[("k", "1"), ("other", "x"), ("k", "2")]);
    assert_eq!(p.get_all("k"), vec!["1", "2"]);
}

#[test]
fn cd_getall_empty_when_absent() {
    let p = URLSearchParams::from_pairs(&[("k", "v")]);
    assert!(p.get_all("missing").is_empty());
}

// ─────────── has (CD URLSearchParams.prototype.has) ──────────

#[test]
fn cd_has_by_name() {
    let p = URLSearchParams::from_pairs(&[("k", "v")]);
    assert!(p.has("k"));
    assert!(!p.has("missing"));
}

#[test]
fn cd_has_with_value_matches_pair() {
    let p = URLSearchParams::from_pairs(&[("k", "v1"), ("k", "v2")]);
    assert!(p.has_with_value("k", "v2"));
    assert!(!p.has_with_value("k", "v3"));
}

// ─────────── set (CD URLSearchParams.prototype.set) ──────────

#[test]
fn cd_set_replaces_all_existing() {
    let mut p = URLSearchParams::from_pairs(&[("k", "1"), ("k", "2"), ("k", "3")]);
    p.set("k", "X");
    assert_eq!(p.get_all("k"), vec!["X"]);
}

#[test]
fn cd_set_preserves_position_of_first() {
    let mut p = URLSearchParams::from_pairs(&[("a", "1"), ("k", "old"), ("b", "2"), ("k", "old2")]);
    p.set("k", "new");
    let entries: Vec<(&str, &str)> = p.entries().collect();
    assert_eq!(entries, vec![("a", "1"), ("k", "new"), ("b", "2")]);
}

#[test]
fn cd_set_appends_when_not_present() {
    let mut p = URLSearchParams::new();
    p.set("k", "v");
    assert_eq!(p.get("k"), Some("v"));
}

// ─────────── sort (CD URLSearchParams.prototype.sort) ──────────

#[test]
fn cd_sort_orders_by_name() {
    let mut p = URLSearchParams::from_pairs(&[("c", "1"), ("a", "2"), ("b", "3")]);
    p.sort();
    let names: Vec<&str> = p.keys().collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn cd_sort_is_stable_within_equal_names() {
    let mut p = URLSearchParams::from_pairs(&[("k", "first"), ("k", "second"), ("k", "third")]);
    p.sort();
    assert_eq!(p.get_all("k"), vec!["first", "second", "third"]);
}

#[test]
fn cd_sort_uses_utf16_code_unit_order() {
    // U+1F600 (a supplementary plane char) has UTF-16 code unit 0xD83D 0xDE00.
    // U+FF21 (full-width A) is 0xFF21. UTF-16 0xD83D < 0xFF21, so emoji-name
    // sorts BEFORE the full-width-A name.
    let mut p = URLSearchParams::from_pairs(&[("\u{FF21}", "z"), ("\u{1F600}", "a")]);
    p.sort();
    let names: Vec<&str> = p.keys().collect();
    assert_eq!(names, vec!["\u{1F600}", "\u{FF21}"]);
}

// ─────────── toString round-trip + spec extras ──────────

#[test]
fn spec_tostring_empty_is_empty() {
    let p = URLSearchParams::new();
    assert_eq!(p.to_string(), "");
}

#[test]
fn spec_tostring_joins_with_ampersand() {
    let p = URLSearchParams::from_pairs(&[("a", "1"), ("b", "2"), ("c", "3")]);
    assert_eq!(p.to_string(), "a=1&b=2&c=3");
}

#[test]
fn spec_tostring_round_trip_through_from_query() {
    let p = URLSearchParams::from_pairs(&[("name with space", "value, with !"), ("k2", "v2")]);
    let serialized = p.to_string();
    let p2 = URLSearchParams::from_query(&serialized);
    assert_eq!(p2.size(), 2);
    assert_eq!(p2.get("name with space"), Some("value, with !"));
    assert_eq!(p2.get("k2"), Some("v2"));
}

// ─────────── Iteration (CD .entries / .keys / .values / .forEach) ──────

#[test]
fn cd_entries_preserves_insertion_order() {
    let p = URLSearchParams::from_pairs(&[("a", "1"), ("b", "2"), ("a", "3")]);
    let entries: Vec<(&str, &str)> = p.entries().collect();
    assert_eq!(entries, vec![("a", "1"), ("b", "2"), ("a", "3")]);
}

#[test]
fn cd_foreach_invokes_for_each_entry() {
    let p = URLSearchParams::from_pairs(&[("a", "1"), ("b", "2")]);
    let mut collected: Vec<(String, String)> = Vec::new();
    p.for_each(|n, v| collected.push((n.to_string(), v.to_string())));
    assert_eq!(collected, vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string())
    ]);
}

// ─────────── size (CD URLSearchParams.prototype.size) ──────────

#[test]
fn cd_size_counts_entries() {
    let p = URLSearchParams::from_pairs(&[("a", "1"), ("a", "2"), ("b", "3")]);
    assert_eq!(p.size(), 3);
}

// ─────────── form-urlencoded edge cases (spec §5.2.5) ──────────

#[test]
fn spec_decode_preserves_invalid_percent_sequences() {
    // Tolerant decoder: "%ZZ" can't decode as hex — preserve literally.
    let p = URLSearchParams::from_query("k=%ZZ");
    assert_eq!(p.get("k"), Some("%ZZ"));
}

#[test]
fn spec_encode_rfc3986_unreserved_punctuation_passthrough() {
    // *, -, ., _ pass through unencoded per the form-urlencoded set.
    let mut p = URLSearchParams::new();
    p.append("k", "*-._");
    assert_eq!(p.to_string(), "k=*-._");
}

#[test]
fn spec_encode_other_ascii_is_percent_encoded() {
    // ! @ # $ % ^ & ( ) etc. all percent-encode.
    let mut p = URLSearchParams::new();
    p.append("k", "!@#$%^&()");
    let s = p.to_string();
    // Must not contain any of these literally:
    for forbidden in &["!", "@", "#", "$", "^", "(", ")"] {
        assert!(!s.contains(forbidden), "must encode {} but got {}", forbidden, s);
    }
    // And must round-trip:
    assert_eq!(URLSearchParams::from_query(&s).get("k"), Some("!@#$%^&()"));
}

#[test]
fn spec_decode_lossy_invalid_utf8_falls_back() {
    // Sequence "%FE%FF" decodes to bytes 0xFE 0xFF which is not valid UTF-8.
    // Spec mandates the decoder does NOT throw; lossy conversion → U+FFFD.
    let p = URLSearchParams::from_query("k=%FE%FF");
    let v = p.get("k").unwrap();
    assert!(v.contains('\u{FFFD}'), "expected U+FFFD replacement, got {:?}", v);
}
