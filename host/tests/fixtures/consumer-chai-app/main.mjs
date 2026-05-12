// chai ^5 — assertion library, pure-ESM. Tests expect DSL + assert API.
import { expect, assert } from "chai";

const lines = [];

function tryAssert(fn) {
  try { fn(); return "pass"; }
  catch (e) { return "fail:" + e.constructor.name; }
}

// 1: expect basics
lines.push("1 equal=" + tryAssert(() => expect(1 + 1).to.equal(2)) +
           " notEqual=" + tryAssert(() => expect(1).to.not.equal(2)) +
           " badEqual=" + tryAssert(() => expect(1).to.equal(2)));

// 2: type assertions
lines.push("2 string=" + tryAssert(() => expect("hi").to.be.a("string")) +
           " number=" + tryAssert(() => expect(42).to.be.a("number")) +
           " badType=" + tryAssert(() => expect("hi").to.be.a("number")));

// 3: deep equality
lines.push("3 deepEq=" + tryAssert(() => expect({a:1, b:[2,3]}).to.deep.equal({a:1, b:[2,3]})) +
           " badDeep=" + tryAssert(() => expect({a:1}).to.deep.equal({a:2})));

// 4: include
lines.push("4 includeArr=" + tryAssert(() => expect([1,2,3]).to.include(2)) +
           " includeStr=" + tryAssert(() => expect("hello").to.include("ell")) +
           " badInclude=" + tryAssert(() => expect([1,2]).to.include(3)));

// 5: throw
lines.push("5 throws=" + tryAssert(() => expect(() => { throw new Error("x"); }).to.throw("x")) +
           " notThrows=" + tryAssert(() => expect(() => { throw new Error("y"); }).to.throw("x")));

// 6: assert.* style
lines.push("6 assertEq=" + tryAssert(() => assert.equal(2 + 2, 4)) +
           " assertOk=" + tryAssert(() => assert.ok(true)) +
           " assertBad=" + tryAssert(() => assert.equal(1, 2)));

// 7: chain length
lines.push("7 lengthOK=" + tryAssert(() => expect([1,2,3]).to.have.lengthOf(3)) +
           " lengthBad=" + tryAssert(() => expect([1,2,3]).to.have.lengthOf(5)));

process.stdout.write(lines.join("\n") + "\n");
