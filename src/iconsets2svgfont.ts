import { createWriteStream, readFileSync } from "node:fs";
import { Readable } from "node:stream";
import { SVGIcons2SVGFontStream } from "svgicons2svgfont";
import { optimize } from "svgo";

const allowList = readFileSync(".allow").toString().split("\n").filter(
  (line) => line,
);

async function getJson(filePath: string) {
  return JSON.parse(await Deno.readTextFile(filePath));
}

for await (const dirEntry of Deno.readDir("./temp/icon-sets")) {
  if (!dirEntry.name.endsWith(".json")) {
    continue;
  }

  const iconSetData = await getJson("./temp/icon-sets/" + dirEntry.name);

  if (allowList.length > 0 && !allowList.includes(iconSetData.prefix)) {
    continue;
  }

  const fontStream = new SVGIcons2SVGFontStream({
    fontName: iconSetData.prefix,
  });
  const codepointsStream = createWriteStream(
    `./temp/fonts/${iconSetData.prefix}.codepoints`,
  ).on("finish", function () {
    console.log(
      `Code points ${iconSetData.prefix}.codepoints successfully created!`,
    );
  })
    .on("error", function (err) {
      console.log(`Error when creating ${iconSetData.prefix}.codepoints`, err);
    });

  // Setting the font destination
  fontStream
    .pipe(createWriteStream(`./temp/fonts/${iconSetData.prefix}.svg`))
    .on("finish", function () {
      console.log(`Font ${iconSetData.prefix}.svg successfully created!`);
    })
    .on("error", function (err) {
      console.log(`Error when creating ${iconSetData.prefix}.svg`, err);
    });

  let codePoint = 0xE000;
  for (const iconName in iconSetData.icons) {
    // optimize SVG with svgo before writing to fontStream
    const result = optimize(
      iconSetData.icons[iconName],
      {
        plugins: [
          "preset-default",
          "mergePaths",
        ],
      },
    );

    const optimizedSvgString = result.data;

    const glyph = Readable.from(optimizedSvgString);

    // @ts-ignore
    glyph.metadata = {
      name: iconName,
      unicode: [String.fromCodePoint(codePoint)],
    };

    fontStream.write(glyph);
    codepointsStream.write(`${iconName} ${codePoint.toString(16)}\n`);

    codePoint++;
  }

  // End streams
  fontStream.end();
  codepointsStream.end();
}
