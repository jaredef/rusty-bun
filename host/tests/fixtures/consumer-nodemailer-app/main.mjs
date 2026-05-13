// nodemailer namespace shape (no SMTP I/O exercised — would need
// network + Tier-G SMTP support).
import nodemailer from "nodemailer";

const transport = nodemailer.createTransport({
  streamTransport: true, newline: "unix", buffer: true,
});

process.stdout.write(JSON.stringify({
  hasCreateTransport: typeof nodemailer.createTransport === "function",
  transportType: typeof transport,
  hasSendMail: typeof transport.sendMail === "function",
}) + "\n");
