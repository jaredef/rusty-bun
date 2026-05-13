import { dequal } from "dequal";
process.stdout.write(JSON.stringify({ eq: dequal({a:1,b:[1,2]}, {a:1,b:[1,2]}) }) + "\n");
