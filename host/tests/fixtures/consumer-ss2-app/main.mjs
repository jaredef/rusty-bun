import { mean, median, mode, standardDeviation, variance, min, max, sum, quantile, linearRegression, linearRegressionLine } from "simple-statistics";

const lines = [];
const xs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

lines.push("1 mean=" + mean(xs) + " median=" + median(xs) + " sum=" + sum(xs));
lines.push("2 min=" + min(xs) + " max=" + max(xs) + " sd=" + standardDeviation(xs).toFixed(4));
lines.push("3 var=" + variance(xs).toFixed(4) + " mode=" + mode([1,2,2,3,3,3,4]));
lines.push("4 q25=" + quantile(xs, 0.25) + " q75=" + quantile(xs, 0.75));

// linear regression
const data = [[1, 2], [2, 4], [3, 6], [4, 8]];
const lr = linearRegression(data);
lines.push("5 m=" + lr.m + " b=" + lr.b);
const f = linearRegressionLine(lr);
lines.push("6 f(5)=" + f(5));

process.stdout.write(lines.join("\n") + "\n");
