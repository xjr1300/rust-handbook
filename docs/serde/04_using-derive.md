# 導出の使用

<https://serde.rs/derive.html>

Serdeは、クレート内に定義されたデータ構造に対して`Serialize`と`Deserialize`の実装を生成する導出マクロを提供しており、Serdeのすべてのデータフォーマットでそれらを便利に表現できるようにします。

**これをするためにコードで`#[derive(Serialize, Deserialize)]`を使用することだけが必要です。**

この機能はRustの`#[derive]`メカニズムに基づいていおり、ビルトインされた`Clone`、`Copy`、`Debug`、または他のトレイトの実装を自動的に導出するために使用されます。
入念なジェネリック型またはトレイト制約をもつものを含め、ほとんどの構造体と列挙体に対して実装を生成することができます。
まれに、特に複雑な型に対しては、[手動でトレイトを実装](https://serde.rs/custom-serialization.html)する必要があるかもしれません。

これらの導出には、Rustコンパイラーバージョン1.31以上を要求します。

- [ ] `Cargo.toml`内の依存関係に`serde = { version = "1.0", features = ["derive"] }`を追加
- [ ] Serdeをベースにした依存関係（例えば、serde_json）が、Serde 1.0と互換性があることを確認
- [ ] シリアライズしたい構造体または列挙型と同じモジュール内に`use serde::Serialize;`として導出マクロをインポートし、構造体または列挙型に`#[derive(Serialize)]`を記述
- [ ] 同様に`user serde::Deserialize;`をインポートし、デシリアライズしたい構造体または列挙型に`#[derive(Deserialize)]`を記述

次に`Cargo.toml`を示します。

```toml
[package]
name = "my-crate"
version = "0.1.0"
authors = ["Me <user@rust-lang.org>"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }

# serde_jsonは単に例で、一般的に要求されません。
serde_json = "1.0"
```

これで、Serdeのカスタム導出を使用する`src/main.rs`は次のようになります。

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = serde_json::to_string(&point).unwrap();
    println!("serialized = {serialized}");

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {deserialized:?}");
}
```

次に出力を示します。

```text
$ cargo run
serialized = {"x",1,"y",2}
deserialized = Point { x: 1, y: 2 }
```

## トラブルシューティング

時々、次を伝えるコンパイル時エラーを見るかもしれません。

```text
the trait `serde::ser::Serialize` is not implemented for `...
```

しかし、それでも構造体または列挙型は、明らかに`#[derive(Serialize)]`があります。

これは、ほとんど常に、互換性のないバージョンのSerdeに依存しているライブラリを使用していることを意味します。
`Cargo.toml`でSerde 1.0に依存しているかもしれませんが、Serde 0.9に依存している他のライブラリを使用しています。
したがって、Serde 1.0からの`Serialize`トレイトが実装されているかもしれませんが、そのライブラリはSerde 0.9の`Serialize`トレイトの実装を予期しています。
Rustコンパイラーの観点から、これらは完全に異なるトレイトです。

修正する方法は、Serdeのバージョンが一致するまで適切にライブラリをアップグレードまたはダウンロードすることです。
`cargo tree -d`コマンドは、入れられている常服した依存関係のすべての場所を探すために便利です。
