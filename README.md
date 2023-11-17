# simplelang

[cordx56](https://github.com/cordx56) 氏の「[Rustで作る！自作言語・コンパイラ入門](https://techbookfest.org/product/z9zCtNAJrigmuu3Jz9VDi?productVariantID=iMxgceXmQkk0T9d3cPskCP&utm_campaign=share&utm_medium=social&utm_source=twitter)」をもとに実装したシンプルな言語のコンパイラです。書籍では JIT コンパイルは LLVM で行っていますが、この実装では代わりに [Cranelift](https://cranelift.dev) を使用しました。

## 使い方

`cargo run` で REPL が起動します。代入文、if 文、print 文の３種類の入力を受け付けます。

以下は使用例です：

```
$ cargo run
   Compiling simplelang v0.1.0 (/path/to/simplelang)
    Finished dev [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/simplelang`
If you want to quit, please enter `quit` or `exit`.
> a = 1
> b = 2
> print a + b
3
> print b - a
1
> if b - a == 1 then print 3
3
> exit
```

