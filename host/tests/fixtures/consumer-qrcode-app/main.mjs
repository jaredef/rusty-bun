try {
  const qrcode = (await import("qrcode")).default;
  process.stdout.write(JSON.stringify({
    hasToString: typeof qrcode.toString === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 100) }) + "\n");
}
