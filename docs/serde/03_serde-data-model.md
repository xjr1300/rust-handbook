# Serdeデータモデル

<https://serde.rs/data-model.html>

Serdeのデータモデルは、データ構造とデータフォーマットが対話するAPIです。
これをSerdeの型システムと考えることができます。

コードにおいて、Serdeデータモデルの半分のシリアライゼーションは[Serializer](https://docs.rs/serde/1/serde/trait.Serializer.html)トレイトによって定義され、半分のデシリアライゼーションは[Deserialize](https://docs.rs/serde/1/serde/trait.Deserializer.html)トレイトによって定義されます。
これらは、Rustデータ構造を29の可能な型から1つにマッピングする方法です。
`Serializer`トレイトのそれぞれのメソッドは、データモデルの型の1つに対応します。

> `Serializer`トレイトは、Rustの基本型、ニュータイプなど用に、次のメソッドなどが定義されている。
>
> - [serialize_bool](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_bool)
> - [serialize_bytes](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_bytes)
> - [serialize_char](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_char)
> - [serialize_i8](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_i8)
> - [serialize_i16](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_i16)
> - ...
> - [serialize_newtype_struct](https://docs.rs/serde/1.0.228/serde/trait.Serializer.html#tymethod.serialize_newtype_struct)
> - ...

データ構造を何らかのフォーマットにシリアライズするとき、そのデータ構造用の`Serialize`の実装は、1つの`Serializer`のメソッドを正確に呼び出すことによって、データ構造とSerdeデータモデルをマッピングする責任がある一方で、データフォーマット用の`Serializer`の実装は、Serdeデータモデルを意図された出力表現にマッピングする責任があります。

何らかのフォーマットからデータ構造をデシリアライズするとき、そのデータ構造用の`Deserializer`の実装は、データモデルの様々な型を受け取ることができる[Visitor](https://docs.rs/serde/1/serde/trait.Deserialize.html)の実装を`Deserializer`に渡すことにより、データ構造をSerdeデータモデルにマッピングする責任がある一方で、データフォーマット用の`Deserializer`の実装は、1つの`Visitor`のメソッドを正確に呼び出すことによって、入力データとSerdeデータモデルをマッピングする責任があります。

## 型

SerdeデータモデルはRustの型システムを簡略化した形式です。
Serdeデータモデルは次の29の型で構成されます。

- **14の基本的な型**
  - `bool`
  - `i8`、`i16`、`i32`,`i64`、`i128`
  - `u8`、`u16`、`u32`,`u64`、`u128`
  - `f32`、`f64`
  - `char`
- **文字列**
  - 長さを持つUTF-8バイト列で`null`で終端されていない文字列、0バイトを含むかもしれません。
  - シリアライズするとき、すべての文字列は等しく処理されます。
    デシリアライズするとき、一時的、所有型、そして借用型の文字列の3つのオプションがあります。
    この区別は[デシリアライザーのライフタイムを理解する](https://serde.rs/lifetimes.html)で説明されており、Serdeは効率的なゼロコピーなデシリアライゼーションをする重要な方法です。
- **バイト配列** - `[u8]`
  - 文字列と類似して、バイト配列をデシリアライズしている間は、一時的、所有型、または借用型になる可能背があります。
- **オプション**
  - `none`または任意の値です。
- **ユニット**
  - Rustの`()`型です。データを含まない匿名の値を表現します。
- **ユニット構造体**
  - 例えば、`struct Unit`または`PhantomData<T>`です。データを含まない名前付きの値を表現します。
- **ユニットバリアント**
  - 例えば、`enum E{ A, B }`のE::A`と`E::B`です。
- **ニュータイプ構造体**
  - 例えば、`struct Millimeters(u8)`です。
- **ニュータイプバリアント**
  - 例えば、`enum E { N(u8) }`の`E::N`です。
- **シーケンス**
  - 可変サイズの異種値のシーケンスで、例えば`Vec<T>`または`HashSet<T>`です。
    シリアライズするとき、すべてのデータを反復処理する前に長さが判明している場合と判明していない場合があります。
    デシリアライズする場合、長さはシリアル化されたデータを参照することで決定されます。
    `vec![Value::Bool(true), Value::Char('c')]`のような同種Rustコレクションは、異種Serdeシーケンスとしてシリアライズされる可能性があることに注意してください。
    この場合、Serdeの`bool`とそれに続くSerdeの`char`が含まれます。
- **タプル**
  - 静的にサイズを持つ異種値のシーケンスで、その長さはシリアライズされたデータを確認することなしで、デシリアライズ時にわかります。
    例えば`(u8,)`または`(String, u64, Vec<T>)`または`[u64; 10]`です。
- **タプル構造体**
  - 名前付きのタプルで、例えば、`struct Rgb(u8, u8, u8)`です。
- **タプルバリアント**
  - 例えば、`enum E { T(u8, u8) }`の`E::T`です。
- **マップ**
  - 可変サイズの異種のキーと値のペアで、例えば`BTreeMap<K, V>`です。
    シリアライズするとき、長さはすべてのエントリを反復操作する前に、わかっているかもしれませんし、わかっていないかもしれません。
    デシリアライズするとき、長さはシリアライズされたデータを確認することで決定されます。
- **構造体**
  - 静的にサイズを持つ異種のキーと値のペアで、キーはコンパイル時に固定文字列で、シリアライズされたデータを確認することなしで、デシリアライズ時にわかります。
    例えば、`struct S { r: u8, g: u8, b: u8 }`です。
- **構造体バリアント**
  - 例えば、`enum E { S { r: u8, g: u8, b: u8 } }`の`E::S`です。

## データモデルにマッピングする

ほとんどのRustの型において、それらをSerdeデータモデルにマッピングすることは単純です。
例えば、Rustの`bool`型は、Serdeの`bool`型に対応しています。
Rustのタプル構造体`struct Rgb(u8, u8, u8)`は、Serdeのタプル構造体型に対応しています。

しかし、これらのマッピングが単純である必要があるという根本的な理由はありません。
`Serialize`と`Deserialize`トレイトは、ユースケースに適したRust型とSerdeデータモデル間の*任意の*マッピングを実行できます。

例として、Rustの[std::ffi::OsString](https://doc.rust-lang.org/std/ffi/struct.OsString.html)型を考えてください。
この型は、プラットフォーム固有の文字列を表現します。
Unixシステムにおいて、それらはゼロでない任意のバイト数で、Windowsシステムにおいてはゼロでない任意の16ビット値です。
`OsString`を次の型の1つをSerdeデータモデルにマッピングすることは、自然に見えるかもしれません。

- Serdeの**文字列**: 残念ながら、`OsString`がUTF-8で表現可能である保証がないため、シリアライゼーションは不安定で、Serdeの文字列はゼロバイトであることを許可されているため、でシリアライゼーションも不安定です。
- Serdeの**バイト配列**: これは、文字列の使用に関する両方の問題を修正しますが、Unixで`OsString`をシリアライズした場合、Windowsでのデシリアライズは[間違った文字列](https://www.joelonsoftware.com/2003/10/08/the-absolute-minimum-every-software-developer-absolutely-positively-must-know-about-unicode-and-character-sets-no-excuses/)に終わります。

`OsString`用の`Serialize`と`Deserialize`実装は、Serdeの**列挙型**として`OsString`を扱うことにより、Serdeデータモデルにマッピングします。
実際のところ、`OsString`型の定義は各プラットフォームでの実装とは一致しないものの、あたかも次のような型として定義されているかのように効果的に振る舞います。

```rust
enum OsString {
    Unix(Vec<u8>),
    Windows(Vec<u16>),
    // そして他のプラットフォーム
}
```

Serdeデータモデルへのマッピング周辺の柔軟さは、奥深く強力です。
`Serialize`と`Deserialize`を実装しているとき、最も直感的なマッピングが最善の選択でない可能性があるため、型の広大なコンテキストに注意してください。
