//! Tier-Ω.5.j.proto: Array.prototype.sort acceptance.

use rusty_js_runtime::{run_module, Value};

fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("run failed for {:?}: {:?}", src, e))
}

fn as_str(v: Value) -> String {
    if let Value::String(s) = v { s.as_str().to_string() } else { panic!("not a string: {:?}", v) }
}

#[test]
fn t1_default_lexicographic() {
    // [10,1,2].sort() => ["1","10","2"] joined "1,10,2".
    assert_eq!(as_str(run("return [10,1,2].sort().join(',');")), "1,10,2");
    assert_eq!(as_str(run("return [3,1,2].sort().join(',');")), "1,2,3");
}

#[test]
fn t2_numeric_ascending_comparator() {
    assert_eq!(as_str(run("return [3,1,2].sort((a,b)=>a-b).join(',');")), "1,2,3");
}

#[test]
fn t3_numeric_descending_comparator() {
    assert_eq!(as_str(run("return [3,1,2].sort((a,b)=>b-a).join(',');")), "3,2,1");
}

#[test]
fn t4_stable() {
    let src = "
        const a = [{k:1,v:'a'},{k:1,v:'b'},{k:2,v:'c'},{k:1,v:'d'}];
        a.sort((x,y)=>x.k-y.k);
        return a.map(o => o.v).join(',');
    ";
    assert_eq!(as_str(run(src)), "a,b,d,c");
}

#[test]
fn t5_empty_array() {
    assert_eq!(as_str(run("return [].sort().join('|');")), "");
}

#[test]
fn t6_mutates_in_place_returns_same() {
    let src = "
        const a = [3,1,2];
        const r = a.sort();
        return (r === a) + ',' + a[0];
    ";
    assert_eq!(as_str(run(src)), "true,1");
}

#[test]
fn t7_strings() {
    assert_eq!(as_str(run("return ['banana','apple','cherry'].sort().join(',');")), "apple,banana,cherry");
}
