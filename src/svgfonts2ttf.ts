import svg2ttf from "npm:svg2ttf";
import { readFileSync, writeFileSync } from "node:fs";
import { Buffer } from "node:buffer";

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

    const svgFontContents = readFileSync(
        `./temp/fonts/${iconSetData.prefix}.svg`,
    );

    const ttf = svg2ttf(svgFontContents.toString(), {});

    writeFileSync(
        `./temp/fonts/${iconSetData.prefix}.ttf`,
        new Buffer(ttf.buffer),
    );

    console.log(`TTF file ${iconSetData.prefix}.ttf successfully created!`);
}
