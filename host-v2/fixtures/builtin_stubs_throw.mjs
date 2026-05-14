// Tier-Ω.5.r: exercises the http.request stub's documented error
// message. The runtime surfaces RuntimeError::TypeError as a host-level
// failure (not a JS-catchable throw in v1), so we drive the binary's
// stderr path to verify the message text.
import http from "node:http";
http.request({}, function () {});
