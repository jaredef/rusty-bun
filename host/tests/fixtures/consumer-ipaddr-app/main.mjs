// ipaddr.js ^2 — IPv4/IPv6 parsing + CIDR matching.
import ipaddr from "ipaddr.js";

const lines = [];

lines.push("1 isV4=" + ipaddr.IPv4.isValid("127.0.0.1") + " badV4=" + ipaddr.IPv4.isValid("999.0.0.1"));
lines.push("2 isV6=" + ipaddr.IPv6.isValid("::1") + " full=" + ipaddr.IPv6.isValid("fe80::1234"));

const a = ipaddr.parse("192.168.1.1");
lines.push("3 kind=" + a.kind() + " octets=" + a.octets.join(","));

const b = ipaddr.parse("::ffff:192.168.1.1");
lines.push("4 v6kind=" + b.kind() + " isV4Mapped=" + b.isIPv4MappedAddress());

const range = ipaddr.IPv4.parseCIDR("10.0.0.0/8");
const inside = ipaddr.IPv4.parse("10.5.6.7").match(range);
lines.push("5 inside=" + inside);

const outside = ipaddr.IPv4.parse("11.0.0.1").match(range);
lines.push("6 outside=" + outside);

process.stdout.write(lines.join("\n") + "\n");
