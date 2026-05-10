// Tier-J consumer #14: binary protocol decoder (spec-first M9 authoring).
//
// In-basin axes not yet exercised:
//   - DataView read/write with little-endian and big-endian
//   - TypedArray subarray + set (zero-copy views)
//   - ArrayBuffer.transfer / .resize (ES2024 may not be in basin — probe via try)
//   - Number.isInteger / .isFinite / .isSafeInteger
//   - Math.hypot, Math.fround, Math.log2, Math.cbrt
//   - Number.prototype.toFixed / .toExponential
//   - Array.prototype.findLast / .findLastIndex (ES2023)
//   - Hex/binary numeric literals (0x... / 0b...)
//   - Float64Array math + reduce

function makeRecord() {
    // Synthetic protocol record:
    //   bytes 0..3   uint32 magic (BE)
    //   bytes 4..5   uint16 version (LE)
    //   bytes 6..7   reserved (zero)
    //   bytes 8..15  float64 latitude (BE)
    //   bytes 16..23 float64 longitude (BE)
    //   bytes 24..27 int32 altitude_mm (LE)
    //   bytes 28..31 uint32 timestamp (LE)
    const buf = new ArrayBuffer(32);
    const view = new DataView(buf);
    view.setUint32(0, 0xCAFEBABE, false);     // BE
    view.setUint16(4, 0x0102, true);          // LE
    view.setUint16(6, 0, true);               // reserved
    view.setFloat64(8, 37.7749, false);       // BE
    view.setFloat64(16, -122.4194, false);
    view.setInt32(24, -150000, true);         // LE, negative
    view.setUint32(28, 0x12345678, true);
    return buf;
}

function decodeRecord(buf) {
    const v = new DataView(buf);
    return {
        magic: v.getUint32(0, false),
        version: v.getUint16(4, true),
        latitude: v.getFloat64(8, false),
        longitude: v.getFloat64(16, false),
        altitude_mm: v.getInt32(24, true),
        timestamp: v.getUint32(28, true),
    };
}

async function selfTest() {
    const results = [];

    // 1. DataView roundtrip with mixed endianness.
    const buf = makeRecord();
    const r = decodeRecord(buf);
    results.push(["dataview-roundtrip",
        r.magic === 0xCAFEBABE &&
        r.version === 0x0102 &&
        r.latitude === 37.7749 &&
        r.longitude === -122.4194 &&
        r.altitude_mm === -150000 &&
        r.timestamp === 0x12345678]);

    // 2. TypedArray subarray creates a view, not a copy.
    const u8 = new Uint8Array(buf);
    const sub = u8.subarray(8, 16);  // latitude bytes
    results.push(["typedarray-subarray",
        sub.length === 8 &&
        sub.buffer === buf &&     // same backing buffer
        sub.byteOffset === 8]);

    // 3. TypedArray.set copies between arrays.
    const a = new Uint8Array([1, 2, 3, 4, 5]);
    const b = new Uint8Array(7);
    b.set(a, 2);
    results.push(["typedarray-set",
        b[0] === 0 && b[1] === 0 && b[2] === 1 && b[6] === 5]);

    // 4. Number predicates.
    results.push(["number-predicates",
        Number.isInteger(42) === true &&
        Number.isInteger(42.5) === false &&
        Number.isFinite(Infinity) === false &&
        Number.isFinite(42) === true &&
        Number.isSafeInteger(2 ** 53) === false &&
        Number.isSafeInteger(2 ** 53 - 1) === true]);

    // 5. Math.hypot for 3D distance from origin.
    const dist = Math.hypot(3, 4, 12);
    results.push(["math-hypot", dist === 13]);

    // 6. Math.fround (single-precision floor of double).
    const f = Math.fround(0.1);  // not exactly 0.1
    results.push(["math-fround",
        typeof f === "number" && Math.abs(f - 0.1) < 1e-7 && f !== 0.1]);

    // 7. Math.log2 / Math.cbrt.
    results.push(["math-extras",
        Math.log2(8) === 3 && Math.cbrt(27) === 3]);

    // 8. toFixed / toExponential formatting.
    const x = 1234.5678;
    results.push(["number-format",
        x.toFixed(2) === "1234.57" &&
        x.toExponential(3) === "1.235e+3"]);

    // 9. Array.findLast / .findLastIndex (ES2023).
    const arr = [1, 3, 5, 4, 6, 2, 8, 7];
    const lastEven = arr.findLast((n) => n % 2 === 0);
    const lastEvenIdx = arr.findLastIndex((n) => n % 2 === 0);
    results.push(["array-findLast",
        lastEven === 8 && lastEvenIdx === 6]);

    // 10. Hex + binary numeric literals.
    results.push(["numeric-literals",
        0xFF === 255 &&
        0b1010 === 10 &&
        0o17 === 15]);

    // 11. Float64Array math via reduce.
    const samples = new Float64Array([1.5, 2.5, 3.5, 4.5]);
    const sum = samples.reduce((s, v) => s + v, 0);
    const mean = sum / samples.length;
    results.push(["float64-reduce",
        sum === 12 && mean === 3]);

    // 12. DataView bounds check throws RangeError.
    let caught = null;
    try {
        new DataView(buf).getUint32(30, false);  // would read past buffer end
    } catch (e) { caught = e; }
    results.push(["dataview-bounds", caught instanceof RangeError]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    process.stdout.write(summary + "\n");
} else {
    globalThis.__esmResult = summary;
}
