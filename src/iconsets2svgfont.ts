import { createWriteStream } from "node:fs";
import { Readable } from "node:stream";
import { SVGIcons2SVGFontStream } from "svgicons2svgfont";
import { optimize } from "svgo";

async function getJson(filePath: string) {
  return JSON.parse(await Deno.readTextFile(filePath));
}

for await (const dirEntry of Deno.readDir("./icon-sets/json")) {
  const iconSetData = await getJson("./icon-sets/json/" + dirEntry.name);

  if (
    ["devicon-plain", "emblemicons"].includes(
      iconSetData.prefix,
    )
  ) {
    continue;
  }

  const fontStream = new SVGIcons2SVGFontStream({
    fontName: iconSetData.prefix,
  });
  const codepointsStream = createWriteStream(
    `fonts/${iconSetData.prefix}.codepoints`,
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
    .pipe(createWriteStream(`fonts/${iconSetData.prefix}.svg`))
    .on("finish", function () {
      console.log(`Font ${iconSetData.prefix}.svg successfully created!`);
    })
    .on("error", function (err) {
      console.log(`Error when creating ${iconSetData.prefix}.svg`, err);
    });

  let codePoint = 0xE000;
  for (const iconName in iconSetData.icons) {
    // FIXME: some icon sets do not define width or height (24 by default?)
    const width = iconSetData.icons[iconName].width ?? iconSetData.width ?? 24;
    const height = iconSetData.icons[iconName].height ?? iconSetData.height ??
      24;

    // optimize SVG with svgo before writing to fontStream
    const result = optimize(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}">${
        iconSetData.icons[iconName].body
      }</svg>`,
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
