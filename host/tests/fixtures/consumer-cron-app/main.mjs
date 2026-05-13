import { CronJob, CronTime } from "cron";
process.stdout.write(JSON.stringify({ hasCronJob: typeof CronJob, hasCronTime: typeof CronTime }) + "\n");
