generate:
	cargo run clean-icon-sets
	deno run iconsets2svgfont
	deno run svgfonts2ttf
	cargo run generate-packages