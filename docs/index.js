"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var pseudoroku_1 = require("pseudoroku");
new pseudoroku_1.PseudoRoku({
    input: "./dialog.txt",
    output: "index.html",
    template: "./template.html",
    profile_lookup: './profile_lookup.tsv',
    getIconPathFromCensoredName: function (name) { return "icons/".concat(name, ".webp"); },
    getMediaPath: function (media) { return "media/".concat(media); },
}).doEverything();
