// Tier-J consumer #36: ECDSA + ECDH over P-384 and P-521.
//
// JWS ES384/ES512, WebAuthn (FIDO2) higher-assurance attestation,
// COSE_Sign1/COSE_Sign with es384/es512 algs, mTLS-with-stronger-curves.
//
// Same coherence pattern as the P-256 fixtures: ECDSA differential
// signal is the summary string (randomized signature), ECDH
// differential signal is byte-equal shared secrets.

const ECDSA_P384 = {
    A: {"kty":"EC","crv":"P-384",
        "x":"HIjdHqNFZ77c4D_77qhr6e8ARYYvvepRx6Z77LFsHz25oU6CXWac1b0F3JyMWXeM",
        "y":"HRCDSXY2YFMkKTklgAAmgwqbI1E8LOtZuDgv610QdHoRGhd7zm6_LKeD9IXrNwz0",
        "d":"_HEGocRAyJJKqwsI8HTYBHsbfy48vvEyu40gY9X8G3tsOV98TfOQW4_VlMcbeJTY"},
    B: {"kty":"EC","crv":"P-384",
        "x":"q1ug-GIvcSeXNr5tdjDD7XQJJAPAjI9uZiW6q1xoMY7DA1Ijzl8mJBNInkmgiAhL",
        "y":"RpRRwaU7d1MZ9X0wZh2gkteZ2_mRX5anEWtsUjzmV3Op_7gfi5CbP5cOnsXPdwQU",
        "d":"0GTG-YBptNUv_ZgXhVlEmArVVoSV_7BeFGd7q6DNgmY3acSoyb8FDNpeRAXl6aVR"},
};
const ECDH_P384 = {
    A: {"kty":"EC","crv":"P-384",
        "x":"HfL73MnzBp2g7rhivtTqLT6v61q1FFty_JyxT0XLj96fekldLeMCDmwZvU3SNwjT",
        "y":"sztPSatiyl5NDZQscT5u0o6y88DeGV-CaoO0YexldQgj5Or-K-SmL3ctei9fW4Cs",
        "d":"mLHXtoeNQLgaCEWem_N1yUQLtf5ohVTz46EZlxwkQqqhJZiUQ3qE67f2-vgR1WjW"},
    B: {"kty":"EC","crv":"P-384",
        "x":"0326rthcFNo4kDihc43aTrIu5V4BfQvecyF85dK3RMW_E0BSU6Nk5Ls7-IJd99nT",
        "y":"VqVnsTwT1TAI46M_YoQDlQyVDA43H-gmo3PACqk_XtSSWKLaxZvh_aGoDyDkeF4P",
        "d":"l8EwDK8ve1euBcEHajOAw3n4MyIQlvuAcJIZZr6dFn1hN0kTRzV0woueM65NQrIz"},
};
const ECDSA_P521 = {
    A: {"kty":"EC","crv":"P-521",
        "x":"AeCBfltJNACZOYdVy80EbUsuVF2Nef-YGrXgODg9TGw3shIeBG0Z8NV90bGA14ZMvynHUMjeebb0ZASuB0gZUZ9v",
        "y":"AS6XWPzeOGKAKaEDPo1ynpdSvW2zSx_Omn82hRMGiSSLY3kr72en33tw0mXSyo7C0AXHA-VZWnq0pBYPqLmW5QEi",
        "d":"AVBGErixWYPmBzP6xqFi4oQA6S-E_EYOI3Va4x5HYRuT_V_5C1yREjXt_NkWmNHpdDk31GRFe3JnkEo0HpahSHuU"},
};
const ECDH_P521 = {
    A: {"kty":"EC","crv":"P-521",
        "x":"AKQAwA-d-YYAQR2ZUBTACm2724dK1uIaUeLnZWb9rY1u54wz7tpBZzg1FXCTpjrhBgzZhgwiF7AFr8c9iZfKfPSg",
        "y":"AN4MsYXT25jWGDzOE6ucT0QDDefFQ-v2pD0ZD3zm8WbRWxR_pNaJGoRmuplYaICzx27SGYNWUlmvItyPTzNKWkrO",
        "d":"AN0l8rEssInyUGu62hAiiFVzNIIfnTbGf38xHWSX_Vb2AYIktVUYLougbfQRrnylncmqSI7Cb75bbmFJz4KJdZIt"},
    B: {"kty":"EC","crv":"P-521",
        "x":"ARBlhXpGtHM8dHC2HRWhumDBtGO1O6rhpATVE7Ey4D8cxTC2QUyeO9eC33adAehVhPPmVky_t8UBNV9N-RQ1iNka",
        "y":"AbYIxERzfVmN8N9nfGNO5Ia0TohqDMla2zUftBGWVRjwzHQq8wW4Q3nTUQZCc1rDEHa5iwNT-HkFeGvfSByiBPl_",
        "d":"AEhTT7XUXhyiZyyYGTRtwwPeUTbGMPQxU-5x0KO-Mue_P-0Gcz5-b0Xvb2q0xaLEu15S52XaC0j6ADnY4LvJFQeP"},
};

function pubOf(j) { return { kty: j.kty, crv: j.crv, x: j.x, y: j.y }; }
function bytesToHex(buf) {
    const arr = buf instanceof ArrayBuffer ? new Uint8Array(buf) : buf;
    let hex = "";
    for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, "0");
    return hex;
}

async function ecdsaCase(name, jwk, hashName, coordBytes) {
    const enc = new TextEncoder();
    const priv = await crypto.subtle.importKey(
        "jwk", jwk.A, { name: "ECDSA", namedCurve: jwk.A.crv }, false, ["sign"]);
    const pub  = await crypto.subtle.importKey(
        "jwk", pubOf(jwk.A), { name: "ECDSA", namedCurve: jwk.A.crv }, false, ["verify"]);
    const msg = enc.encode("msg-" + name);
    const sig = await crypto.subtle.sign({ name: "ECDSA", hash: hashName }, priv, msg);
    const ok = await crypto.subtle.verify({ name: "ECDSA", hash: hashName }, pub, sig, msg);
    const wrongOk = await crypto.subtle.verify(
        { name: "ECDSA", hash: hashName }, pub, sig, enc.encode("tampered"));
    return sig.byteLength === 2 * coordBytes && ok === true && wrongOk === false;
}

async function ecdhCase(name, jwk, coordBytes) {
    const aPriv = await crypto.subtle.importKey(
        "jwk", jwk.A, { name: "ECDH", namedCurve: jwk.A.crv }, false, ["deriveBits"]);
    const aPub  = await crypto.subtle.importKey(
        "jwk", pubOf(jwk.A), { name: "ECDH", namedCurve: jwk.A.crv }, false, []);
    const bPriv = await crypto.subtle.importKey(
        "jwk", jwk.B, { name: "ECDH", namedCurve: jwk.B.crv }, false, ["deriveBits"]);
    const bPub  = await crypto.subtle.importKey(
        "jwk", pubOf(jwk.B), { name: "ECDH", namedCurve: jwk.B.crv }, false, []);
    const fromA = await crypto.subtle.deriveBits({ name: "ECDH", public: bPub }, aPriv, coordBytes * 8);
    const fromB = await crypto.subtle.deriveBits({ name: "ECDH", public: aPub }, bPriv, coordBytes * 8);
    return fromA.byteLength === coordBytes && bytesToHex(fromA) === bytesToHex(fromB);
}

async function selfTest() {
    const results = [];
    results.push(["ecdsa-p384-sha384", await ecdsaCase("p384", ECDSA_P384, "SHA-384", 48)]);
    results.push(["ecdh-p384",         await ecdhCase("p384", ECDH_P384, 48)]);
    results.push(["ecdsa-p521-sha512", await ecdsaCase("p521", ECDSA_P521, "SHA-512", 66)]);
    results.push(["ecdh-p521",         await ecdhCase("p521", ECDH_P521, 66)]);
    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
