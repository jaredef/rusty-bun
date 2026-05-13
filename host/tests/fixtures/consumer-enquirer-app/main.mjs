import enquirer from "enquirer";
process.stdout.write(JSON.stringify({ hasPrompt: typeof enquirer.prompt }) + "\n");
