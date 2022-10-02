import { PseudoRoku } from 'pseudoroku';

new PseudoRoku({
	input: `./../dialog.txt`,
	output: `index.html`,
	template: `./template.html`,
	profile_lookup: './profile_lookup.tsv',
	getIconPathFromCensoredName: name => `icons/${name}.webp`,
	getMediaPath: media => `media/${media}`,
}).doEverything();

