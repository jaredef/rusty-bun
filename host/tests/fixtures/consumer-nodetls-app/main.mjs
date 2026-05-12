import tls from "node:tls";
import * as tlsNamed from "node:tls";

const out = {
  hasConnect: typeof tls.connect === "function",
  hasTLSSocket: typeof tls.TLSSocket === "function",
  hasRootCerts: Array.isArray(tls.rootCertificates),
  minVer: tls.DEFAULT_MIN_VERSION,
  maxVer: tls.DEFAULT_MAX_VERSION,
  hasCheck: typeof tls.checkServerIdentity === "function",
  namedHasConnect: typeof tlsNamed.connect === "function",
  socketShape: (() => {
    const s = new tls.TLSSocket();
    return ["on", "write", "end", "destroy", "setEncoding", "getCipher", "getPeerCertificate"]
      .map(m => typeof s[m] === "function");
  })(),
};
process.stdout.write(JSON.stringify(out) + "\n");
