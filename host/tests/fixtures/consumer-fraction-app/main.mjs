// fraction.js ^5 — rational arithmetic (distinct from decimal/bignumber).
import Fraction from "fraction.js";

const lines = [];

lines.push("1 " + new Fraction(1, 2).add(new Fraction(1, 3)).toFraction());
lines.push("2 " + new Fraction(5, 6).sub("1/2").toFraction());
lines.push("3 " + new Fraction("3/4").mul("8/9").toFraction());
lines.push("4 " + new Fraction(7, 2).div(new Fraction(1, 4)).toString());
lines.push("5 " + new Fraction(0.25).toFraction());
lines.push("6 " + new Fraction("0.(3)").toFraction());

process.stdout.write(lines.join("\n") + "\n");
