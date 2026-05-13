import Color from "color";

const red = Color("red");
const blue = Color.rgb(0, 0, 255);
const mix = red.mix(blue, 0.5);

process.stdout.write(JSON.stringify({
  redHex: red.hex(),
  blueHex: blue.hex(),
  mixHex: mix.hex(),
  redArr: red.array(),
  hsl: red.hsl().array(),
}) + "\n");
