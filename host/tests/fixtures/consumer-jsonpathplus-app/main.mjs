import { JSONPath } from "jsonpath-plus";

const data = {
  store: {
    book: [
      { category: "fiction", author: "Tolkien", title: "LOTR", price: 22.99 },
      { category: "fiction", author: "Le Guin", title: "Earthsea", price: 8.95 },
      { category: "reference", author: "Knuth", title: "TAOCP", price: 99 },
    ],
    bicycle: { color: "red", price: 19.95 },
  },
};

const all = JSONPath({ path: "$..price", json: data });
const titles = JSONPath({ path: "$.store.book[*].title", json: data });
const fiction = JSONPath({ path: "$..book[?(@.category=='fiction')].author", json: data });
const first = JSONPath({ path: "$..book[0]", json: data });

process.stdout.write(JSON.stringify({
  all,
  titles,
  fiction,
  firstAuthor: first[0] && first[0].author,
}) + "\n");
