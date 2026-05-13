import chalk from "chalk";
process.stdout.write(JSON.stringify({ hasRed: typeof chalk.red, hasBold: typeof chalk.bold }) + "\n");
