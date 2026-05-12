// decimal.js ^10 — arbitrary-precision decimal arithmetic. Distinct
// math axis. Tests basic ops + precision-preserving operations that
// would lose accuracy under native f64.
import Decimal from "decimal.js";

const lines = [];

// 1: simple add (vs f64's 0.1 + 0.2 = 0.30000000000000004)
{
  const r = new Decimal("0.1").plus("0.2");
  lines.push("1 r=" + r.toString() + " eq=" + r.eq("0.3"));
}

// 2: large multiplication
{
  const r = new Decimal("12345678901234567890").times("98765432109876543210");
  lines.push("2 " + r.toString());
}

// 3: division precision
{
  const r = new Decimal("1").div("3");
  // Default 20 sig figs
  lines.push("3 " + r.toString());
}

// 4: power
{
  const r = new Decimal("2").pow(10);
  lines.push("4 2^10=" + r.toString());
}

// 5: comparison
{
  const a = new Decimal("100");
  const b = new Decimal("99.999");
  lines.push("5 gt=" + a.gt(b) + " lt=" + b.lt(a) + " eq=" + a.eq("100.0"));
}

// 6: sqrt
{
  const r = new Decimal("2").sqrt();
  // First 10 digits of sqrt(2)
  lines.push("6 sqrt2=" + r.toFixed(10));
}

// 7: precision control + rounding mode
{
  Decimal.set({ precision: 5 });
  const r = new Decimal("1").div("7");
  Decimal.set({ precision: 20 });  // reset
  lines.push("7 " + r.toString());
}

// 8: NaN + Infinity
{
  const inf = new Decimal("1").div("0");
  const nan = new Decimal("0").div("0");
  lines.push("8 inf=" + inf.toString() + " nan=" + nan.toString() + " isFinite=" + inf.isFinite() + " isNaN=" + nan.isNaN());
}

process.stdout.write(lines.join("\n") + "\n");
