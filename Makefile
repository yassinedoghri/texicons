generate:
	cargo run clean-icon-sets
	deno run iconsets2svgfont
	deno run svgfonts2ttf
	cargo run generate-packages

clear:
	find ./temp/icon-sets ! -name '.gitkeep' -type f -exec rm -f {} +
	find ./temp/fonts ! -name '.gitkeep' -type f -exec rm -f {} +

submodules-update:
	git submodule foreach git pull