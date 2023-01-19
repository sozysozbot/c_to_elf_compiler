# c_to_elf_compiler
Follows https://www.sigbus.info/compilerbook but spits out an ELF / Rui 本ベースで ELF を吐く C コンパイラを作る

[作業ログはこちら](https://sozysozbot.github.io/c_to_elf_compiler/log.html)

## 発表
[Kernel/VM探検隊online part6](https://kernelvm.connpass.com/event/256278/) で発表しました

- [スライド](https://docs.google.com/presentation/d/1ms7ZhaBbwB9zPcvax-ElwFXIMu28jE4D6QRBWNPoIDo)
- [録画](https://youtu.be/yuSVbuiaBuU?t=12063)

## Plan
[hsjoihs](https://twitter.com/hsjoihs) will implement the odd-numbered steps, and [kgtkr](https://twitter.com/kgtkr) will implement the even-numbered steps. / hsjoihs が奇数番目のステップを、kgtkr が偶数番目のステップを実装する方針

## Files and folders
- `log.txt`: conversations and logs in plaintext (written entirely in Japanese) / 会話および作業ログがテキスト形式で記録されている
- `docs/`: `log.txt` is rendered nicely into the [GitHub Page](https://sozysozbot.github.io/c_to_elf_compiler/log.html), and the files in `docs/` are for generating this web page. / `log.txt` をいい感じにレンダリングしたやつが [GitHub Page](https://sozysozbot.github.io/c_to_elf_compiler/log.html) で閲覧できる。`docs/` 内のファイルはこのページを生成するためのもの。
- `experiment/`: a folder intended for experimenting how gcc works and so on / gcc の挙動とかを実験するためのフォルダ
- `src/`: a folder in which the actual C compiler's source lives / C コンパイラのソースが入っているフォルダ
