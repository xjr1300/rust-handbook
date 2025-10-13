# no-stdサポート

`serde`クレートは、デフォルトで有効にされる`"std"`と呼ばれるCargoフィーチャーがあります。
`no_std`コンテキストでSerdeを使用するために、このフィーチャを無効にしなければなりません。
Cargo.toml内のSerde依存関係を、デフォルトによって有効なフィーチャを選択肢ないように、修正してください。

```toml
[dependencies]
serde = { version = "1.0", default-features = false }
```

Cargoフィーチャーが依存グラフ全体で統合されていることに気をつけてください。
これは、Serdeのデフォルトフィーチャを選択している他のクレートに依存している場合、Serdeへの直接の依存が`default-features = false`であるかないかにかかわらず、`std`フィーチャーを有効にしてSerdeをビルドすることを意味します。

例えば、`serde_json`を使用している場合、それはデフォルトで`"std"`を有効にすると等価であるため、それで無効にする必要があります。

```toml
serde = { version = "1.0", default-features = false }
serde_json = { version = "1.0", default-features = false, features= ["alloc"] }
```

## 導出

`#[derive(Serialize, Deserialize)]`[導出マクロ](https://serde.rs/derive.html)は、`no_std`クレートでも同様に機能します。

```toml
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

ヒープに割り当てられた一時的なバッファを必要とするデシリアライゼーション機能は、メモリアロケーターなしで、`no_std`モードで利用できません。
特に、[タグなし列挙型](https://serde.rs/enum-representations.html)はデシリアライズされません。

## メモリ割り当て

Serdeの`"std"`を選択しないことは、`String`と`Vec<T>`を含む、ヒープメモリ割り当てを巻き込む標準ライブラリのデータ構造のサポートを削除します。
また、タグなし列挙型を含む、`derive(Deserialize)`の機能もいくつか削除します。

`"alloc"`Cargoフィーチャを有効にすることで、これらの実装を選択し直すことができます。
この構成は、Rust標準ライブラリの残りに依存することなしで、ヒープに割り当てられるコレクション用の統合を提供します。

```toml
[dependencies]
serde = { version = "1.0", default-features = false, features = ["alloc"] }
```
