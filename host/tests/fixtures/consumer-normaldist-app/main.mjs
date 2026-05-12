import NormalDistribution from "normal-distribution";

const nd = new NormalDistribution(0, 1);
const nd2 = new NormalDistribution(10, 2.5);
const out = {
  pdfAt0: nd.pdf(0).toFixed(6),
  pdfAt1: nd.pdf(1).toFixed(6),
  pdfAt_neg2: nd.pdf(-2).toFixed(6),
  cdfAt0: nd.cdf(0).toFixed(6),
  cdfAt1: nd.cdf(1).toFixed(6),
  cdfAt_neg2: nd.cdf(-2).toFixed(6),
  prob_neg1_to_1: nd.probabilityBetween(-1, 1).toFixed(6),
  prob_neg2_to_2: nd.probabilityBetween(-2, 2).toFixed(6),
  nd2_pdfAt10: nd2.pdf(10).toFixed(6),
  nd2_cdfAt12: nd2.cdf(12).toFixed(6),
};
process.stdout.write(JSON.stringify(out) + "\n");
