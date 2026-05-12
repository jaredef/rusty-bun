// semver ^7 — version parser ubiquitous in npm. Pure-JS CJS through the
// CJS-in-ESM bridge. Tests parse, compare, satisfies, valid ranges,
// inc, diff, coerce.
import semver from "semver";

const lines = [];

// 1: parse + valid
lines.push("1 valid1.2.3=" + semver.valid("1.2.3") + " validBad=" + semver.valid("not-a-version"));

// 2: compare
lines.push("2 lt=" + semver.lt("1.2.3", "1.2.4") + " gt=" + semver.gt("2.0.0", "1.9.9") + " eq=" + semver.eq("1.0.0", "1.0.0"));

// 3: satisfies range
lines.push("3 caret=" + semver.satisfies("1.2.5", "^1.2.0") +
           " tilde=" + semver.satisfies("1.2.5", "~1.2.0") +
           " range=" + semver.satisfies("1.5.0", ">=1.2.0 <2.0.0"));

// 4: increment
lines.push("4 patch=" + semver.inc("1.2.3", "patch") +
           " minor=" + semver.inc("1.2.3", "minor") +
           " major=" + semver.inc("1.2.3", "major") +
           " pre=" + semver.inc("1.2.3", "prerelease", "alpha"));

// 5: diff
lines.push("5 diff(1.2.3, 1.2.4)=" + semver.diff("1.2.3", "1.2.4") +
           " diff(1.2.3, 2.0.0)=" + semver.diff("1.2.3", "2.0.0") +
           " diff(1.2.3, 1.2.3)=" + semver.diff("1.2.3", "1.2.3"));

// 6: coerce
lines.push("6 coerce('v2')=" + (semver.coerce("v2") || {}).version +
           " coerce('1.0')=" + (semver.coerce("1.0") || {}).version +
           " coerce('garbage')=" + (semver.coerce("garbage")));

// 7: prerelease detection
lines.push("7 pre=" + JSON.stringify(semver.prerelease("1.2.3-alpha.1")) +
           " stableNoPre=" + (semver.prerelease("1.2.3") === null));

// 8: max/min satisfying
lines.push("8 max=" + semver.maxSatisfying(["1.0.0", "1.2.0", "1.3.0", "2.0.0"], "^1.0.0") +
           " min=" + semver.minSatisfying(["1.0.0", "1.2.0", "1.3.0", "2.0.0"], "^1.0.0"));

process.stdout.write(lines.join("\n") + "\n");
