# カスタムなシリアライゼーション

<https://serde.rs/custom-serialization.html>

Serdeの`#[derive(Serialize, Deserialize)]`を介した[導出マクロ](https://serde.rs/derive.html)は、構造体と列挙型に対して妥当なデフォルトのシリアライゼーションの振る舞いを提供して、[属性](https://serde.rs/attributes.html)を使用することである程度カスタマイズできます。
通常でない使用では、Serdeは型に対して[Serialize](https://docs.rs/serde/1/serde/ser/trait.Serialize.html)と[Deserialize](https://docs.rs/serde/1/serde/de/trait.Deserialize.html)を手作業で実装することで、シリアライゼーションの振る舞いを完全にカスタマイズできます。

トレイトそれぞれは1つのメソッドを持っています。

```rust
pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}
```

これらのメソッドはシリアライゼーションフォーマットに対してジェネリックで、`Serializer`と`Deserializer`トレイトによって表現されます。
例えば、JSONに対しては1つのシリアライザーがあり、Postcardに対しては異なる1つがあります。

## Serializeの実装

`Serialize`トレイトは次のとおりです。

```rust
pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}
```

このメソッドの仕事は、型を受け取り（`&self`）、指定された`Serializer`のメソッドの1つを正確に呼び出して、それを[Serdeデータモデル](https://serde.rs/data-model.html)にマッピングします。

ほとんどの場合、Serdeの`derive`はクレート内で定義された構造体と列挙型に対して適切な`Serialize`の実装を生成します。
`derive`がサポートしていない方法で、型に独自なシリアライゼーションの振る舞いが必要な場合、自身で`Serialize`を実装できます。

### プリミティブのシリアライゼーション

最も単純な例として、次にプリミティブな`i32`に対するビルトインされた`Serialize`の実装を示します。

```rust
impl Serialize for i32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(*self)
    }
}
```

SerdeはRustのすべての[プリミティブ型](https://doc.rust-lang.org/book/primitive-types.html)に対してこのような実装を提供しているため、自身でそれらを実装する責任はありませんが、`serialize_i32`とそれに似たようなメソッドは、シリアライズされた書式がプリミティブとして表現される必要がある型がある場合、役に立つかもしれません。
例えば、[プリミティブな数値としてC言語のような列挙型をシリアライズ](https://serde.rs/enum-number.html)できます。

### シーケンスやマップのシリアライゼーション

合成型は初期化、イテレーション（`elements`）、終了の3つのステップの処理に従います。

```rust
user serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    let mut seq = serializer.serialize_seq(Some(self.len()))?;
    for e in self {
        seq.serialize_element(e)?;
    }
    seq.end()
}

impl<K, V> Serialize for MyMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    let mut map = serializer.serialize_map(Some(self.len()))?;
    for (k, v) in self {
        map.serialize_entry(k, v)?;
    }
    map.end()
}
```

### タプルのシリアライゼーション

`serialize_tuple`メソッドは、`serialize_seq`と多くの点が似ています。
その違いは、`serialize_tuple`は、長さがデシリアライズ時にわかるため、長さをシリアライズする必要がないシーケンス用であることです。
通常の例は、Rustの[タプル](https://doc.rust-lang.org/std/primitive.tuple.html)と[配列](https://doc.rust-lang.org/std/primitive.array.html)です。
非自己説明フォーマットにおいて、`Vec<T>`は、`Vec<T>`にデシリアライズできるようにするために、その長さをシリアライズする必要があります。
しかし、`[T; 16]`は、シリアライズされたバイト数を確認することなしに、その長さをデシリアライズ時に知ることができるため、`serialize_tuple`でシリアライズできます。

### 構造体のシリアライズ

Serdeは構造体の4つの種類を区別します。
[普通の構造体](https://doc.rust-lang.org/book/structs.html)と[タプル構造体](https://doc.rust-lang.org/book/structs.html#tuple-structs)は、シーケンスやマップのように、初期化、反復（`elements`）、終了の3つのステップに従います。
[ニュータイプ構造体](https://doc.rust-lang.org/book/structs.html#tuple-structs)と[ユニット構造体](https://doc.rust-lang.org/book/structs.html#unit-like-structs)はよりプリミティブに似ています。

```rust
// 普通の構造体で、次の3つのステップを使用
//  1. serialize_struct
//  2. serialize_field
//  3. end
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

// タプル構造体で、次の3つのステップを使用
//  1. serialize_tuple_struct
//  2. serialize_field
//  3. end
struct Point2D(f64, f64);

// ニュータイプ構造体では、serialize_newtype_structを使用
struct INches(u64);

// ユニット構造体では、serialize_unit_structを使用
struct Instance;
```

構造体とマップは、JSONを含みいくつかのフォーマットで似ているかもしれません。
その区別は、構造体がシリアライズされたデータを確認しないで、デシリアライズしているときに、コンパイル時に固定の文字列であるキーを持つことです。
この条件は、構造体を処理するいくつかのフォーマットで、マップよりも効率的かつ小さくできます。

データフォーマットは、内部の値をラップする無意味なラッパーとしてニュータイプ構造体を扱い、内部の値だけをシリアライズすることが推奨されます。
[JSONのニュータイプ構造体の取り扱い](https://serde.rs/json.html)を参照してください。

```rust
use serde::ser::{Serialize, Serializer, SerializeStruct};

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Serializer for Color {
    fn serialize<S>(&self, serializer: S) ->Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 工事体内の3つのフィールの値
        let mut state = serializer.serialize_struct("Color", 3);
        state.serialize_field("r", &self.r)?;
        state.serialize_field("g", &self.g)?;
        state.serialize_field("b", &self.b)?;
        state.end()
    }
}
```

### 列挙型のシリアライズ

列挙型のバリアントのシリアライズは、構造体のシリアライズと非常に似ています。

```rust
enum E {
    // 3ステップの処理を使用
    //  1. serialize_struct_variant
    //  2. serialize_field
    //  3. end
    Color { r: u8, g: u8, b: u8 },

    // 3ステップの処理を使用
    //  1. serialize_tuple_variant
    //  2. serialize_field
    //  3. end
    Point2D(f64, f64),

    // serialize_new_type_variantを使用
    Inches(u64),

    // serialize_unit_variantを使用
    Instance,
}
```

### その他特別なケース

`Serializer`トレイトの部分で2つのより特別なケースがあります。

`&[u8]`をシリアライズする`serialize_bytes`メソッドがあります。
いくつかのフォーマットはバイト列を他の任意のシーケンスとして取り扱いますが、いくつかのフォーマットはバイト列をより小さくシリアライズできます。
現在、Serdeは、`&[u8]`または`Vec<u8>`の`Serialize`実装に`serialize_bytes`を使用していませんが、[特殊化](https://github.com/rust-lang/rust/issues/31844)が安定したRustに取り込まれたら、それを使用する予定です。
現時点では、[serde_bytes](https://docs.rs/serde_bytes)クレートは`serialize_bytes`を介して`&[u8]`と`Vec<u8>`の処理を効率的にするために使用できます。

最後に、`serialize_some`と`serialize_none`は　`Option::Some`と`Option::None`に対応してきました。
ユーザーは、`Option`列挙型に対して他の列挙型とは異なる想定を持っている傾向があります。
`serde_json`は`Option::None`を`null`、`Option::Some`を単に含んだ値としてシリアライズしています。

## Deserializeの実装

[Deserialize](https://docs.rs/serde/1/serde/de/trait.Deserialize.html)トレイトは次です。

```rust
pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}
```

このメソッドの役割は、型のインスタンスを構築する`Deserializer`によって起動する[Visitor](https://docs.rs/serde/1/serde/de/trait.Visitor.html)を使用する[Deserializer](https://docs.rs/serde/1/serde/trait.Deserializer.html)を提供することで、型を[Serdeデータモデル](https://serde.rs/data-model.html)にマッピングすることです。

ほとんどのケースでは、Serdeの[derive](https://serde.rs/derive.html)は、クレートに定義された構造体と列挙型に対して`Deserialize`の適切な実装を生成することができます。
`derive`がサポートしていない方法で、型に独自なデシリアライゼーションの振る舞いが必要な場合、自分で`Deserialize`を実装できます。
型に`Deserialize`を実装することは、`Serialize`よりも複雑になる傾向があります。

`Deserializer`トレイトは、異なるデシリアライゼーションを可能にする2つのエントリポイントのスタイルをサポートします。

**1. `deserialize_aty`メソッド**

JSONのような自己説明データフォーマットはシリアライズされたデータを確認して、それが表現しているものを伝えることができます。
例えば、JSONデシリアライザーは、左波括弧（`{`）を確認したら、マップを見ていることに気付くことができます。
もし、データフォーマットが`Deserialize::deserialize_any`をサポートしている場合、任意のJSONドキュメントを表現できる列挙型である`serde_json::Value`をデシリアライズしているとき、入力された型に基づいて`Visitor`を駆動します。
何のJSONドキュメントなのか知らないで、`Deserializer_::deserialize_any`を介して、それを`serde_json::Value`にデシリアライズできます。

**2. いろいろな`deserialize_*`メソッド**

Postcardのような非自己説明データフォーマットは、それをデシリアライズするために入力が何であるかを伝える必要があります。
`deserialize_*`メソッドは入力の次の一変を解釈する方法をデシリアライザーにヒントを与えます。
非自己説明フォーマットは、`Deserializer::deserialize_any`に依存する`serde_json::Value`のようなものをデシリアライズできません。

`Deserialize`を実装しているとき、入力内のものが何の型かデシリアライザーに伝えることなしで、`Deserializer::deserialize_any`に依存することを避けるべきです。
`Deserializer::deserialize_any`に依存することは、データ型が自己説明フォーマットだけからデシリアライズできることを意味して、Postcardや多くの他のフォーマットがルール外であることを理解してください。

### Visitorトレイト

[Visitor](https://docs.rs/serde/1/serde/de/trait.Visitor.html)は`Deserializer`の実装によってインスタンス化され、`Deserializer`に渡されます。
その後、`Deserializer`は、望まれた型を構築するために`Visitor`のメソッドを呼び出します。

様々な型からプリミティブな`i32`をデシリアライズできる｀Visitor`を次に示します。

```rust
use std::fmt;

use serde::de::{self, Visitor};

struct I32Visitor;

impl<'de> Visitor<'de> for I32Visitor {
    type Value = i32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(i32::from(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::i32;
        if value >= i64::from(i32::Min) && value <= i64::from(i32::MAX) {
            Ok(value as i32)
        } else{
            Err(E::custom(format!("i32 out of range: {}", value)))
        }
    }

    // 他のメソッドも同様
    //  - visit_i16
    //  - visit_u8
    //  - visit_u16
    //  - visit_u32
    //  - visit_u64
}
```

`Visitor`トレイトは、`I32Visitor`が実装していないより多くのメソッドがあります。
それらを実装しないままで残すことは、それらが呼ばれた場合に[型エラー](https://docs.rs/serde/1/serde/de/trait.Error.html#method.invalid_type)が返されることを意味します。
例えば、`I32Visitor`は`Visitor::visit_map`を実装していないため、入力にマップが含まれているときに`i32`をデシリアライズすることを試みると型エラーになります。

### Visitorをドライビングする

`Visitor`を与えられた`Deserializer`に渡すことで値をデシリアライズします。
`Deserializer`は入力データに基づいて`Visitor`の1つのメソッドを呼び出し、それは`Visitor`のドライビングとして知られています。

```rust
impl<'de> Deserialize<'de> for i32 {
    fn deserialize<D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i32(I32Visitor)
    }
}
```

`Deserializer`が必ず型ヒントに従う必要はないため、`deserialize_i32`を呼び出しても、`Deserializer`が`I32Visitor::visit_i32`を呼び出すことを意味しないことに注意してください。
例えば、JSONは似たようなすべての符号付き整数型を取り扱います。
JSONの`Deserializer`は、ヒントが異なる型でも、任意の整数型のために`visit_i64`を呼び出し、そして`visit_64`は符号付き整数用です。

### その他の例

- [マップのデシリアライズ](https://serde.rs/deserialize-map.html)
- [構造体のデシリアライズ](https://serde.rs/deserialize-struct.html)

## ユニットテスト

[serde_test](https://docs.rs/serde_test)クレートは、`Serialize`と`Deserialize`の実装に対してユニットテストを記述する便利で簡潔な方法を提供しています。

値に対する`Serialize`実装は、値をシリアライズする過程で行われる`Serializer`の呼び出しの列によってキャラクタライズ（符号化）されるため、`serde_test`は`Serializer`のメソッド呼び出しにほぼ対応する[Token](https://docs.rs/serde_test/1/serde_test/enum.Token.html)抽象化を提供します。
`serde_test`は、値が特定のメソッド呼び出しの列でシリアライズされるかをテストする機能を提供する`assert_ser_tokens`関数、値が特定のメソッド呼び出しの列からデシリアライズされることをテストする`assert_de_tokens`関数、そして両方向をテストする`assert_tokens`関数を提供します。
さらに、`serde_test`は、予期された失敗条件をテストする関数も提供しています。

次は、[linked-hash-map](https://github.com/contain-rs/linked-hash-map)クレートにある例です。

```rust
use linked_hash_map::LinkedHashMap;
use serde_test::{Token, assert_tokens};

#[test]
fn test_ser_de_empty() {
    let map = LinkedHashMap::<char, u32>::new();

    assert_tokens(&map, &[
        Token::Map { len: Some(0) },
        token::MapEnd,
    ]);
}

#[test]
fn test_ser_de() {
    let mut map = LinkedHashMap::new();
    map.insert('b', 20);
    map.insert('a', 10);
    map.insert('c', 30);

    assert_tokens(&map, &[
        Token::Map { len: Some(3) },
        Token::Char('b'),
        Token::I32(20),

        Token::Char('a'),
        Token::I32(10),

        Token::Char('c'),
        Token::I32(30),
        Token::MapEnd,
    ])
}
```
