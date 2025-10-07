# Serdeの概要

<https://serde.rs/>

Serdeは、効率的かつ一般的にRustデータ構造をシリアライズまたはデシリアライズするフレームワークです。

Serdeエコシステムは、どのようにシリアライズまたはデシリアライズするのかを知っているデータ構造と、どのように他のものをシリアライズまたはデシリアライズするのかを知っているデータフォーマットで構成されます。
Serdeは、これら2つのグループが互いに相互作用するためのレイヤを提供し、サポートされた任意のデータ構造がサポートされた任意のデータフォーマットを使用してシリアライズまたはデシリアライズできるようにします。

![Decrusting the serde crate](https://youtu.be/BI_bHCGRgMY)

## 設計

他の多くの言語が、データをシリアライズするためにランタイムで行うリフレクションに依存していますが、Serdeは代わりにRustの強力なトレイトシステムで構築されています。
自身をシリアライズまたはデシリアライズする方法を知っているデータ構造は、Serdeの`Serialize`と`Deserialize`トレイトを実装しているものです（またはコンパイル時に実装を自動的に生成するSerdeの導出属性を使用して）。
これは、リフレクションの負荷または、ランタイムの型情報を避けます。
実際、多くの状況において、データ構造とデータフォーマット間のやり取りは、SerdeのシリアライズがRustコンパイラーによって完全に最適化される可能性があり、特定のデータ構造とデータフォーマットを選択した場合、手動実装したシリアライザーと同じ速度で実行されます。

## データフォーマット

次は、コミュニティによってSerde用に実装されたデータフォーマットのリストの一部です。

- [JSON](https://github.com/serde-rs/json): 多くのHTTP APIで利用されるどこでも存在するJavaScript Object Notation
- [Postcard](https://github.com/jamesmunns/postcard): `no_std`で組み込みシステムと親和性の高いコンパクトなバイナリフォーマット
- [CBOR](https://github.com/enarx/ciborium): バージョン交渉の必要がない小さなメッセージサイズで設計されたConcise Binary Object Representation
- [YAML](https://github.com/dtolnay/serde-yaml): マークアップ源と出ない、自称人間に優しい設定言語
- [MessagePack](https://github.com/3Hren/msgpack-rust): 簡潔なJSONに似た効率的なバイナリフォーマット
- [TOML](https://docs.rs/toml): [Cargo](https://doc.rust-lang.org/cargo/reference/manifest.html)で使用される最小の設定フォーマット
- [Pickle](https://github.com/birkenfeld/serde-pickle): Pythonの世界で一般的なフォーマット
- [RON](https://github.com/ron-rs/ron): RustのObject Notation
- [BSON](https://github.com/mongodb/bson-rust): MongoDBで使用されるデータストレージとネットワーク転送フォーマット
- [Avro](https://docs.rs/apache-avro): スキーマ定義をサポートしたApache Hadoopで使用されるバイナリフォーマット
- [JSON5](https://github.com/callum-oakley/json5-rs): ES5由来のいくつかのプロダクトを含んだJSONの上位集合
- [URL](https://docs.rs/serde_qs): `x-www-form-urlencoded`フォーマット内のクエリ文字列
- [Starlark](https://github.com/dtolnay/serde-starlark): BazelとBuckビルドシステムによってビルドターゲットを指定するためのフォーマット（シリアライズのみ）
- [Envy](https://github.com/softprops/envy): Rust構造体内に環境変数をデシリアライズする方法（デシリアライズのみ）
- [Envy Store](https://github.com/softprops/envy-store): Rust構造体内に[AWS Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)パラメーターをデシリアライズする方法
- [S-expressions](https://github.com/rotty/lexpr-rs): Lisp言語ファミリーで使用されるコードとデータのテキスト表現
- [D-Bus](https://docs.rs/zvariant) D-Busのバイナリフォーマット
- [FlexBuffers](https://github.com/google/flatbuffers/tree/master/rust/flexbuffers): GoogleのFlatBufferのスキーマのないバージョンで、ゼロコピーなシリアライズフォーマット
- [Bencode](https://github.com/P3KI/bendy): BitTorrentプロトコルで使用される単純なバイナリフォーマット
- [Token streams](https://github.com/oxidecomputer/serde_tokenstream): Rustの宣言的マクロの入力を処理（デシリアライズのみ）
- [DynamoDB items](https://docs.rs/serde_dynamo): DynamoDB間でデータを転送するために[rusoto_dynamodb](https://docs.rs/rusoto_dynamodb)で使用されるフォーマット
- [Hjson](https://github.com/Canop/deser-hjson): 人間が読んだり編集できるように設計されたJSONの構文拡張（デシリアライズのみ）
- [CSV](https://docs.rs/csv): 表形式のテキストファイルフォーマットであるカンマで分割された値

## データ構造

箱から取り出して、Serdeは上記の任意のフォーマットに一般的なRustデータ型をシリアライズまたはデシリアライズできます。
例えば、`String`、`&str`、`usize`、`Vec<T>`、`HashMap<K, V>`はすべてサポートされています。
加えて、Serdeは独自のプログラム内の構造体にシリアライズ実装を生成する導出マクロを提供しています。
導出マクロの使用方法は、次のようになります。

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // PointをJSON文字列に変換
    let serialized = serde_json::to_string(&point).unwrap();

    // serialized = {"x":1,"y":2} を出力
    println!("serialized = {serialized}");

    // JSON文字列をPointに戻す
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // deserialized = Point { x: 1, y: 2 } を出力
    println!("deserialized = {deserialized:?}");
}
```
