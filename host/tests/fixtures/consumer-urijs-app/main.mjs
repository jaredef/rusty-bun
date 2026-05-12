import * as URI from "uri-js";

const lines = [];

// 1: parse URI
{
  const p = URI.parse("https://user:pass@host.example.com:8080/path/to/resource?q=1&k=v#fragment");
  lines.push("1 scheme=" + p.scheme + " host=" + p.host + " port=" + p.port + " path=" + p.path + " query=" + p.query + " fragment=" + p.fragment + " userinfo=" + p.userinfo);
}

// 2: resolve relative
{
  lines.push("2 " + URI.resolve("http://example.com/a/b/c", "../d/e"));
}

// 3: normalize
{
  lines.push("3 " + URI.normalize("HTTPS://Example.COM/A/./B/../C/"));
}

// 4: serialize round-trip
{
  const parsed = URI.parse("https://example.com/test?x=1");
  const back = URI.serialize(parsed);
  lines.push("4 back=" + back);
}

// 5: IPv6
{
  const p = URI.parse("http://[2001:db8::1]:80/");
  lines.push("5 host=" + p.host);
}

process.stdout.write(lines.join("\n") + "\n");
