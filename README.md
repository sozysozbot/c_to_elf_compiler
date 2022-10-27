# c_to_elf_compiler
Follows https://www.sigbus.info/compilerbook but spits out an ELF / Rui 本ベースで ELF を吐く C コンパイラを作る

[作業ログはこちら](https://sozysozbot.github.io/c_to_elf_compiler/log.html)

## Plan
[hsjoihs](https://twitter.com/hsjoihs) will implement the odd-numbered steps, and [kgtkr](https://twitter.com/kgtkr) will implement the even-numbered steps. / hsjoihs が奇数番目のステップを、kgtkr が偶数番目のステップを実装する方針

## Files and folders
- `log.txt`: conversations and logs in plaintext (written entirely in Japanese) / 会話および作業ログがテキスト形式で記録されている
- `docs/`: `log.txt` is rendered nicely into `docs/log.html`, which can be seen on the [GitHub Page](https://sozysozbot.github.io/c_to_elf_compiler/log.html). All the rest are for generating this file. I know this whole machinery should be living in a CI, but I'm too lazy to set that up. / `log.txt` をいい感じにレンダリングしたやつが `docs/log.html`。これは [GitHub Page](https://sozysozbot.github.io/c_to_elf_compiler/log.html) で閲覧できる。残りのファイルはこの `docs/log.html` を生成するためのもの。本来は CI でここら辺を構築すべきなのだろうが、めんどいのでやらないでおく。
- `experiment/`: a folder intended for experimenting how gcc works and so on / gcc の挙動とかを実験するためのフォルダ
- `src/`: a folder in which the actual C compiler's source lives / C コンパイラのソースが入っているフォルダ
