import { PseudoRoku } from 'pseudoroku';

new PseudoRoku({
	input: `./../log.txt`,
	output: `log.html`,
	template: `./template.html`,
	profile_lookup: './profile_lookup.tsv',
	getIconPathFromCensoredName: name => `icons/${name}.webp`,
	getMediaPath: media => `media/${media}`,
}).doEverything();

