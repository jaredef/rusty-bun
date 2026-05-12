// Prettier loads and exposes format/check/etc. Calling format() hits
// prettier's dynamic-eval-of-ESM-source path (plugin loader), which
// requires module-context eval — a separate substantial edge. This
// fixture verifies load + namespace shape; full format/check exercises
// are deferred until module-context dynamic eval lands.
import prettier from "prettier";

process.stdout.write(JSON.stringify({
  hasFormat: typeof prettier.format === "function",
  hasCheck: typeof prettier.check === "function",
  hasFormatWithCursor: typeof prettier.formatWithCursor === "function",
  hasDoc: typeof prettier.doc === "object",
}) + "\n");
