// mongoose — ODM. Load + namespace shape (full DB connection requires
// MongoDB protocol + driver beyond rusty-bun scope).
import mongoose from "mongoose";

const Schema = mongoose.Schema;
const userSchema = new Schema({ name: String, age: Number });

process.stdout.write(JSON.stringify({
  hasSchema: typeof mongoose.Schema === "function",
  hasModel: typeof mongoose.model === "function",
  hasConnect: typeof mongoose.connect === "function",
  hasMongo: typeof mongoose.Mongoose === "function",
  schemaHasPath: typeof userSchema.path === "function",
}) + "\n");
