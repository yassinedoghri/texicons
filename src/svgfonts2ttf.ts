import svg2ttf from "npm:svg2ttf";
import { readFileSync, writeFileSync } from "node:fs";
import { Buffer } from "node:buffer";

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

    const svgFontContents = readFileSync(`./fonts/${iconSetData.prefix}.svg`);

    const ttf = svg2ttf(svgFontContents.toString(), {});

    writeFileSync(`./fonts/${iconSetData.prefix}.ttf`, new Buffer(ttf.buffer));
}
