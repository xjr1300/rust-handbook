# データフォーマットの記述

<https://serde.rs/data-format.html>

データフォーマットを記述する前に理解するべき最も重要なことは、**Serdeが解析ライブラリでない**ことです。
Serdeに実装しているフォーマットが何であるか解析することを支援しません。
Serdeの役目はとても特有です。

- **シリアライゼーション**: ユーザーから任意のデータ構造を受け取り、最大限の効率性でそれらをフォーマットにレンダリングします。
- **デシリアライゼーション**: ユーザーが選択したデータ構造に解析するデータを最大限の効率性で解釈します。

解析はこれらのどちらでもなく、初めから解析するコードを記述するか、デシリアライザーを実装するために解析ライブラリを使用するかどちらかです。

理解するべき2つ目に重要なことは[Serdeデータモデル](https://serde.rs/data-model.html)です。

## 慣例

慣例により、Serdeのデータフォーマットクレートは、ルートモジュールで次を提供するか、ルートモジュールから再エクスポートされて提供されます。

- シリアライゼーションとデシリアライゼーションの両方で一般的なエラー型
- `std::result::Result<T, Error>`と同等な`Result`型の定義
- `serde::Serializer`を実装した`Serializer`型
- `serde::Deserializer`を実装した`Deserializer`型
- フォーマットがシリアライズをサポートする型に応じた1つ以上の`to_abc`関数
  - 例えば、`String`を返す`to_string`、`Vec<u8>`を返す`to_bytes`、または[io::Write](https://doc.rust-lang.org/std/io/trait.Write.html)に書き込む`to_writer`
- フォーマットがデシリアライズをサポートする型に応じた1つ以上の`from_xyz`関数
  - 例えば、`&str`を受け取る`from_str`、`&[u8]`を受け取る`from_bytes`、または[io::Read](https://doc.rust-lang.org/std/io/trait.Read.html)を受け取る`from_reader`

加えて、シリアライザーとデシリアライザーを超えて、シリアライゼーション特有またはデシリアライゼーション特有のAPIを提供するフォーマットは、それらを最上位の`ser`及び`de`モジュールの下に公開しなければなりません。
例えば、`serde_json`は[serde_json::ser::Formatter](https://docs.rs/serde_json/1/serde_json/ser/trait.Formatter.html)として、自由に取り付け可能な整形した出力を提供しています。

基本的なデータフォーマットは次のように始まります。
3つのモジュールはより詳細は次のページで議論します。

```rust
// src/lib.rs
mod de;
mod error;
mod ser;

pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_string, Serializer};
```

## エラー処理

シリアライゼーションの間、`Serialize`トレイトはRustデータ構造とSerdeのデータモデルをマッピングして、`Serializer`トレイトはデータモデルと出力フォーマットをマッピングします。
デシリアライゼーションの間、`Deserializer`は入力データをSerdeのデータモデルにマッピングして、`Deserialize`と`Visitor`トレイトはデータモデルと結果となるデータモデルをマッピングします。
これらのステップは失敗する可能性があります。

- `Serialize`は、例えば、`Mutex<T>`がシリアライズされ、ミューテックスは毒に侵されると失敗する可能性があります。
- `Serializer`は、例えば、Serdeデータモデルが文字列以外のキーを持つマップは許可されますが、JSONは許可されないため、失敗する可能性があります。
- `Deserializer`は特に入力データが構文的に不正な場合、失敗する可能性があります。
- `Deserialize`は、通常、デシリアライズしようとする値に対して入力が誤った型の場合、失敗する可能性があります。

Serdeにおいて、`Serializer`と`Deserializer`からのエラーは、他のRustライブラリのそれらと似ています。
クレートがエラー型を定義して、この関数がエラー型を含む`Result`を返し、色々な可能性のある失敗モードに対するバリアンtのがあります。

ライブラリによって処理されるデータ構造である`Serialize`と`Deserialize`からのエラーの処理は、[ser::Error](https://docs.rs/serde/1/serde/ser/trait.Error.html)と[de::Error](https://docs.rs/serde/1/serde/de/trait.Error.html)トレイトを中心に構築されます。
これらのトレイトは、データフォーマットに、いろいろな状況で使用するデータ構造用のエラー型のコンストラクタを公開することを許可します。

```rust
// src/errors.rs
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

// これは必要最低限​​の機能を備えた実装です。実際のライブラリでは、エラーの種類に応じて、
// 例えばエラーが発生した行と列、入力のバイトオフセット、現在処理中のキーなど、
// 追加情報が提供されます。
#[derive(Debug)]
pub enm Error {
    // `ser::Error`と`de::Error`トレイトを介したデータ構造によって作成される1つ以上のバリアントです。
    // 例えば、Mutex<T>に対するSerializeの実装が、ミューテックスが毒に侵されていることが理由で、
    // エラーを返すかもしれず、構造体に対するDeserializerの実装が、要求されるフィールドが足りないために、
    // エラーを返すかもしれません。
    Message(String),

    // `ser::Error`と`de::Error`を介さないで、シリアライザーとデシリアライザーによって、直接作成された
    // ゼロまたはより多くのバリアントです。これは特定のフォーマット特有で、この場合はJSONです。
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            // など　
        }
    }
}

impl std::error::Error for Error {}
```

## シリアライザーの実装

このページは、Serdeを使用して基本的ですが、JSONシリアライザーの基本的な実装を提供します。

`Serializer`トレイトは多くのメソッドがありますが、この実装内のひとつも複雑ではありません。
それぞれのメソッドは、Serdeのデータモデルの1つの型に対応しています。
シリアライザーは、データモデルを出力表現にマッピングする責任があり、この場合はJSONです。

それぞれのメソッドの使用方法の例については、`Serializer`トレイトのドキュメントを参照してください。

> `Serialize`は、自分のデータをどのようにシリアライズするかを定義する。
> したがって、型に対して`Serialize`を実装する。
>
> `Serializer`は、どのような形式に変換するかを定義する。
>
> ```rust
> pub trait Serialize {
>     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
>     where
>         S: Serializer;
> }
> ```
>
> - `S`は、どのようなフォーマットに変換するかを決める`Serializer`型である。
> - `self.serialize(serializer)`を呼び出すと、`self`が`serializer`にデータを渡す。
>
> 次の実装において、`&'a mut Serializerに対して、可変参照を`self`として受け取るそれぞれのデータ型を
> シリアライズするメソッドを実装していることに注意すること。
> `self`は可変参照であるため、それぞれのメソッドで可変参照自体は消費されるが、その参照先の実体は消費されない。
>
> 合成型のシリアライズは、開始、要素の反復処理、終了の3ステップで行われる。
> 開始は`Serializer`の実装が担い、要素の反復処理と終了は`SerializeSeq`など、それぞれの合成型の実装が担う。

```rust
// src/ser.rs
use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub struct Serializer {
    // この文字列は殻で始まり、JSONがシリアライズされた値として追加されます。
    output: String,
}

// 慣例により、Serdeシリアライザーの公開APIは、`to_string`、`to_bytes`、または`to_writer`
// のような1つ以上の関数で、シリアライザーが出力を生成できるRustの型に依存しています。
//
// この基本的なシリアライザーは、`to_string`のみサポートしています。
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // この`Serializer`がシリアライゼーションを成功したときに生成する出力型。
    // テキストまたはバイナリ出力を生成するほとんどのシリアライザーは、ここで行われているように、
    // `Ok = ()`を設定して、`io::Write`または`Serializer`インスタンスを含むバッファー
    // にシリアライズするべきです。
    // メモリ内にデータ構造を構築するシリアライザーは、データ構造周辺に伝播する`Ok`を使用する
    // ことて、簡素化されるかもしれません。
    type Ok = ();

    // シリアライズの間にエラーが発生したときのエラー型。
    type Error = Error;

    // シーケンスやマップのような合成したデータ構造をシリアライズしている間に追加的な状態を追跡し続けるための関連型。
    // この場合、Serializer構造体内に既に蓄積されているものがなんであっても、追加的な状態は要求されません。
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // 簡単な方法を説明します。
    // 次の12メソッドは、データモデルのプリミティブな型の1つを受け取り、出力文字列に追加することで、それをJSONに
    // マッピングします。
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    // JSONは整数の異なるサイズを区別しないため、すべての符号付き整数は同じようにシリアライズされ、すべての符号なし
    // 整数は同じようにシリアライズされます。他のフォーマットでは、特に小さなバイナリフォーマットでは、異なるサイズに
    // 対して独立なロジックが必要かもしれません。
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    // 特に効率的ではありませんが、とにかくこれはコード例です。
    // より性能が良い方法は、`itoa`クレートを使用することかもしれません。
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string()
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f64) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    // 1文字の文字列としてcharをシリアライズします。
    // 他のフォーマットはこれと異なる表現かもしれません。
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // これは、エスケープシーケンスを要求しない文字列に対してのみ機能しますが、その意味はわかるでしょう。
    // 例えば、入力文字列が'"'文字を保つ場合、不正なJSONを出力するでしょう。
    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    // バイトの配列としてバイト配列をシリアライズします。
    // これはbase64文字列でも使用できます。
    // バイナリフォーマットは、通常より小さくバイト配列を表現できます。
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // 存在しないオプションはJSONで`null`として表現されます。
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // 存在するオプションは単に含んだ値を表現します。
    // これは損失のある表現であることに注意してください。
    // 例えば、値`Some(())`と`None`は両方とも単に`null`にシリアライズされます。
    // 残念ながら、これは通常、JSONを扱うときに人々が期待するものです。
    // 他のフォーマットでは、可能であればより賢く振る舞うように推奨されています。
    fn serialize_some<T>(self, value: T) -> Result<()>
    where
        T: Serialize + ?Send,
    {
        value.serialize(self)
    }

    // Serdeにおいて、ユニットはデータのない匿名の値を意味します。
    // これをJSONの`null`にマップします。
    fn serialize_unit(self) -> Result<()> {
        self.output += "null";
        Ok(())
    }

    // ユニット構造体はデータのない名前付きの値を意味します。
    // 再度、データが無いため、これをJSONの`null`にマップします。
    // ほとんどのフォーマットで、名前をシリアライズする必要はありません。
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // ユニットバリアント（または他の種類のバリアント）をシリアライズしているとき、フォーマットはインデックスまたは名前により
    // それを追跡し続けるかを選択できます。
    // バイナリーフォマットでは通常、バリアントのインデックスが使用され、人間が読めるフォーマットでは通常、名前が使用されます。
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // ここで行っているように、シリアライザーは、ニュータイプ構造体が含んでいるデータの重要でないラッパーとして扱うことが
    // 推奨されています。
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    // ニュータイプバリアント（と他のすべてのバリアントのシリアライゼーションメソッド）は、排他的に「外部タグ付き」列挙型表現
    // を参照します。
    //
    // これを`{ NAME: VALUE }`のように、外部タグ付けされた形式でJSONにシリアライズします。
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":";
        value.serialize(&mut *self)?;
        self.output += "}";
        Ok(())
    }

    // ここで、合成型のシリアライゼーションを得ます。
    //
    // シーケンスの開始、それぞれの値、そして終了は、3つの異なるメソッド呼び出しです。
    // このひとつはシリアライゼーションの開始のみの役割があり、それはJSONにおける`[`です。
    //
    // シーケンスの長さは前もって知っているかもしれませんし、知らないかもしれません。
    // これは、シリアライズされた形式で明示的に長さが表現されないため、JSONでは違いがありません。
    // あるシリアライザーは、長さを事前に知ることでシーケンスをサポートできます。
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";
        Ok(self)
    }

    // JSONにおいてタプルはシーケンスのように見えます。
    // あるフォーマットは長さを省略することでタプルをより効率的に表現できるかもしれないため、タプルは
    // 対応する`Deserializer`の実装がシリアライズされたデータを確認する必要なく、長さを知ることが
    // できる可能性があります。
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // JSONにおいてタプルバリアントは`{ NAME: [DATA...] }`として表現されます。
    // 再度、このメソッドは外部タグ付き表現に対してのみ責任があります。
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":[";
        Ok(self)
    }

    // JSONにおいてマップは`{ K: V, K: V, ... }`として表現されます。
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "{";
        Ok(self)
    }

    // JSONにおいて構造体はマップのように見えます。
    // 特に、JSONは構造体のフィールド名をシリアライズすることを要求します。
    // 他のフォーマットは、対応するデシリアライズの実装が、シリアライズされたデータを確認することなく、
    // キーを知ることを要求するため、構造体をシリアライズしているとき、フィールド名を省略する可能性が
    // あります。
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    // JSONにおいて構造体のバリアントは`{ NAME: {K: V, ... } }`のように表現されます。
    // これは外部タグ付き表現です。
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":{";
        Ok(self)
    }
}

// 次の7つの実装は、シーケンスやマップのような合成型のシリアライゼーションを扱います。
// そのような型のシリアライゼーションは、シリアライザーメソッドとによって開始され、合成型の個々の要素を
// シリアライズするために零以上のメソッドの呼び出しが続き、そして合成型を終了するための1つの呼び出しに
// なります。
//
// この実装はSerializeSeqで、これらのメソッドはシリアライザーの`serialize_seq`を呼び出した後に
// 呼び出されます。
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // シリアライザーの`Ok`型にマッチしなければなりません。
    type Ok = ();
    // シリアライザーの`Error`型にマッチしなければなりません。
    type Error = Error;

    // シーケンスのひとつの要素をシリアライズします。
    //
    // selfは、&mut &'a mut Serializer型であるため、シリアライズメソッドでは2重に参照外しを
    // して実体を取得した後、その可変参照を取得している。
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // シーケンスを閉じます。
    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

// 同じですがタプルに対する実装です。
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

// 同じですがタプル構造体に対する実装です。
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

// タプルバリアントは少し異なります。
// 上記の`serialize_tuple_variant`メソッドに戻って確認してください。
//
//      self.output += "{";
//      variant.serialize(&mut **self)?;
//      self.output += ":[";
//
// この実装内の`end`メソッドは、`]`と`}`の両方を閉じる責任があります。
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]}";
        Ok(())
    }
}

// ある`Serialize`型は同時にメモリ内にキーと値を保持することができないため、`SerializeMap`の実装は
// `serialize_key`と`serialize_value`それぞれのサポートを要求します。
//
// `SerializeMap`トレイトには、3番目のオプションメソッドが存在します。
// `serialize_entry`メソッドは、キーと値の両方が同時に利用可能な場合に、シリアライザーを最適化できます。
// JSONでは違いがないため、`serialize_entry`のデフォルトの振る舞いで問題ありません。
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // Serdeデータモデルは、キーを任意のシリアライズ可能な型にマップできます。
    // JSONは文字列のキーのみを許可するため、下の実装は、もしキーが文字列でなく他の何かにシリアライズした場合、
    // 不正なJSONを生成します。
    //
    // 実際のJSONシリアライザーは、マップのキーが文字列であることを検証する必要があるかもしれません。
    // これは、キーをシリアライズするために（`&mut **self`ではなく）、`serialize_str`のみを実装して、他のデータ型でエラーを返す
    // 他のシリアライザーを使用することで行うことができます。
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)
    }

    // コロンが`serialize_key`の末尾または`serialize_value`の開始に出力されるかどうかは、
    // 違いがありません。
    // この場合、コードはここに記載すると少し簡単になります。
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }

    // 構造体はコンパイル時にキーが定数文字列に制限されるマップのようなものです。
    impl<'a> ser::SerializeStruct for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where
            T: Serialize + ?Sized,
        {
            if !self.output.ends_with('{') {
                self.output += ",";
            }
            key.serialize(&mut **self)?;
            self.output += ":";
            value.serialize(&mut **self)
        }

        fn end(self) -> Result<()> {
            self.output += "}";
            Ok(())
        }
    }
}

// `SerializeTupleVariant`に似て、ここの`end`メソッドは`serialize_struct_variant`によって
// 開かれた両方の波括弧を閉じる役目があります。
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialized + ?Sized,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}}";
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let expected = r#"{"int":1,"seq":["a","b"]}"#;
    assert_eq!(to_string(&test).unwrap(), expected);
}

#[test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    let expected = r#""Unit""#;
    assert_eq!(to_string(&u).unwrap(), expected);

    let n = E:Newtype(1);
    let expected = r#"{"Newtype":1}"#;
    assert_eq!(to_string(&n).unwrap(), expected);

    let t = E::Tuple(1, 2);
    let expected = r#"{"Tuple":[1,2]}"#;
    assert_eq!(to_string(&t).unwrap(), expected);

    let s = E::Struct { a: 1};
    let expected = r#"{"Struct":{"a":1}}"#;
    assert_eq!(to_string(&s).unwrap(), expected);
}
```

## デシリアライザーの実装

このページでは、Serdeを使用したJSONデシリアライザーの基本的な、しかし機能的な実装を示します。

シリアライザーと同様に、[Deserializer](https://docs.rs/serde/1/serde/de/trait.Deserializer.html)トレイトには多くのメソッドがありますが、この実装内ではどれも複雑ではありません。
デシリアライザーは、シリアライザーが受け取る[Visitor](https://docs.rs/serde/1/serde/de/trait.Visitor.html)のひとつのメソッドを正確に呼び出すことにより、入力データを[Serdeデータモデル](https://serde.rs/data-model.html)にマッピングする役目があります。

`Deserializer`メソッドは、`Deserializer`型が入力に期待するSerdeデータモデル型をヒントとして、`Deserialize`の実装から呼び出されます。
JSONのような自己説明フォーマットでは、このヒントを`Deserializer`が無視して、入力データの内容に対応する`Visitor`メソッドを呼び出しても問題ありません。
他のフォーマットにおいて、特にPostcardのような小さなバイナリフォーマットは、入力データが解釈される方法を決定するヒントに頼ります。

自己説明フォーマットは、ヒントを無視して、`Deserializer`トレイトのメソッドの一部またはすべてを、`deserialize_any`メソッドに転送する[forward_to_deserialize_any!](https://docs.rs/serde/1/serde/macro.forward_to_deserialize_any.html)マクロを使用することで多くコードを記述しなくても良いようにできます。

下のコードは、ドキュメント目的で明示的にすべてのメソッドを実装していますが、それに利点はありません。

[デシリアライザーのライフタイム](https://serde.rs/lifetimes.html)は、それ自身専用のページがあります。

```rust
// src/de.rs
use std::opt::{AddAssign, MulAssign, Neg};

use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess,SeqAccess,
    VariantAccess, Visitor,
};

use error::{Error, Result};

pub struct Deserializer<'de> {
    // この文字列は、入力データで始まり、データが解析されるにつれて、先頭の文字列が切り捨てられます。
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    // 慣例により、`Deserializer`コンストラクターは`from_xyz`のように名前が付けられます。
    // そうすれば、`serde_json::from_str(...)`のような基本的なユースケースは満たされ、デシリアライザー
    // が要求される高度なユースケースは`serde_json::Deserializer::from_str(...)`でデシリアライザーを
    // 作成できます。
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

// 慣例により、Serdeデシリアライザーの公開APIは、デシリアライザーが入力として消費できるRustの型に応じて、
// `from_str`、`from_bytes`、または`from_reader`のような、ひとつ以上のメソッドになります。
//
// 基本的なデシリアライザーは`from_str`のみをサポートします。
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

// Serdeは解析ライブラリではありません。
// この実装ブロックは、最初からいくつか基本的な解析関数を定義しています。
// より複雑なフォーマットは、それらのSerdeデシリアライザーの実装を助ける専用の解析ライブラリを使用することを
// 希望するかもしれません。
impl<'de> Deserializer<'de> {
    // 消費せずに入力内の最初の文字を確認します。
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    // 入力の最初の文字を消費します。　
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    // JSONの識別子`true`または`false`を解析します。
    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("true") {
            self.input = &self.input["true".len()..];
            Ok(true)
        } else if self.input.starts_with("false") {
            self.input = &self.input["false".len()..];
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    // T型の符号なし整数として、10進数の数字のグループを解析します。
    //
    // この実装は少し緩すぎて、例えば`001`はJSONで許可されません。
    // また、色々な算術操作はオーバーフローして、パニックまたは偽りのデータを返す可能性があります。
    // しかし、コード例としては十分です！
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_char()? {
            ch @ '0'..='9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input_chars.next() {
                Some(ch @ '0'..'9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as u8 - b'0');
                }
                _ => {
                    return Ok(int);
                }
            }
        }
    }

    // マイナス記号の後に続く10進数のグループを、T型の符号付き整数として解析します。　
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        // マイナス記号はオプションで、`parse_unsigned`に委譲して、もじマイナスであれば、マイナスにします。
        //
        // 下の`deserialize_any`メソッドの実装を確認すると、`-`を見たとき、デシリアライズを`deserialize_i64`が行っている。
        // したがって、マイナスの可能性のある10進数を解析するこのメソッドは呼び出されない。
        unimplemented!()
    }

    // 次が`"`文字になるまで文字列を解析します。
    //
    // エスケープシーケンスを処理しません。何を予想しますか？これはコード例です！
    fn parse_string(&mut self) -> Result<&'de str> {
        if self.next_char()? != '"' {
            return Err(Error::ExpectedString);
        }
        match self.input.find('"') {
            Seme(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len + 1...];
                Ok(s)
            }
            None => Err(Error::Eof),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // デシリアライズするSerdeデータモデルの型を決定するために入力データを確認します。
    // すべてのフォーマットがこの操作をサポートできません。
    // `deserialize_any`をサポートするフォーマットは自己説明として知られています。
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_char()? {
            'n' => self.deserialize_unit(visitor),
            't' | 'f' => self.deserialize_bool(visitor),
            '"' => self.deserialize_str(visitor),
            '0'..='9' => self.deserialize_u64(visitor),
            '-' => self.deserialize_i64(visitor),
            '[' => self.deserialize_seq(visitor),
            '{' => self.deserialize_map(visitor),
            _ => Err(Error::Syntax),
        }
    }

    // 上で定義された`parse_bool`解析関数は、入力からJSONの識別子`true`または`false`を読み込むために使用されます。
    //
    // 解析とは、入力を確認して、それがJSONの`true`または`false`を含むか決定することです。
    //
    // デシリアライズとは、`Visitor`の1つのメソッドを呼び出すことで、JSONの値をSerdeデータモデルにマッピングすることです。
    // JSONと`bool`の場合、マッピングは簡単なので、区別は馬鹿げているかも知れませんが、他の場合、デシリアライザーは時々、明らかでないマッピングを
    // 実行します。
    // 例えば、TOMLフォーマットは日時型があり、Serdeデータモデルはありません。
    // `toml`クレートにおいて、入力の日時型は、特別な名前を持つSerdeデータモデルの「構造体」型にマッピングすることで、、文字列として表現された
    // 日時を含んでいる単一のフィールドにデシリアライズされます。
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // `parse_signed`関数は整数型`T`に対してジェネリックなため、ここでは`T=i8`で呼び出されます。
    // 次の8つのメソッドは似ています。　
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    // 小数点数の解析はとても難しいです。
    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // 小数点数の解析はとても難しいです。
    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // 前のページの`Serializer`実装では、文字を1文字の文字列としてシリアル化したため、ここでその表現を処理します。
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // 文字列を解析して、それが1文字であることを確認するために、`visit_char`を呼び出します。
        unimplemented!()
    }

    // Serdeにおける文字列の3つのデシリアライゼーションの種類についての情報を「デシリアライザーのライフタイムを理解する」ページで参照してください。
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // バイト配列をJSONのバイト配列としてシリアライズした前ページの実装を、ここでその表現を処理します。
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // 省略可能要素がない場合、JSONの`null`として表現され、省略可能要素がある場合、省略可能要素は、その要素に含まれる値として表現されます。
    //
    // `Serializer`の実装でコメントされているように、これは非可逆的な表現です。
    // 例えば、値`Some(())`と`None`はどちらも単なる`null`としてシリアライズされます。
    // 残念ながら、これはJSONを扱う際に一般的に予想される動作です。
    // 他の形式では、可能であればより賢い動作が推奨されます。
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    // Serdeに置いて、ユニットはデータを含まない匿名の値を意味します。
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    // ユニット構造体は、データを含まない名前を付けられた値です。
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // ここで行われているように、シリアライザはnewtype構造体を、それに含まれるデータの無意味なラッパーとして扱うことが推奨されます。
    // これは、含まれる値以外は何も解析しないということを意味します。
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // シーケンスやマップなどの複合型のデシリアライズは、ビジターに「Access」オブジェクトを渡すことで実行されます。
    // これにより、シーケンスに含まれるデータを反復処理できるようになります。
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // シーケンスの開始カギ括弧を解析します。
        if self.next_char()? == '[' {
            // シーケンスのそれぞれの要素にアクセスするビジターを与えます。
            let value = visitor.visit_seq(CommaSeparated::new(self))?;
            // シーケンスの終了カギ括弧を解析します。
            if self.next_char()? == ']' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    // タプルはJSONのシーケンスと全く同じように見えます。一部の形式ではタプルをより効率的に表現できる場合があります。
    //
    // `length`パラメータで示されているように、Serdeデータモデルのタプルに対する`Deserialize`実装では、
    // 入力データを見る前にタプルの長さを知る必要があります。
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // タプル構造体はJSONのシーケンスのように見えます。
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // `deserialize_seq`とよく似ていますが、ビジターの`visit_seq`メソッド（`SeqAccess`実装）ではなく、ビジターの
    // `visit_map`メソッド（`MapAccess`実装）を呼び出します。
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // マップの開始波カッコを解析します。
        if self.next_char()? == '{' {
            // ビジターにマップのそれぞれのエントリにアクセスさせます。
            let value = visitor.visit_map(CommaSeparated::new(self))?;
            // マップの終了波カッコを解析します。
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    // 構造体はJSONにおけるマップのように見えます。
    //
    // `fields`パラメータに注目してください。
    // Serdeデータモデルにおける「構造体」とは、入力データを見る前に、`Deserialize`実装がフィールドの内容を
    // 把握する必要があることを意味します。
    // フィールドが事前に分からないキーと値のペアは、おそらくマップです。
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_char()? == '"' {
            // ユニットバリアントを訪ねます。
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if self.next_char()? == '{' {
            // ニュータイプバリアント、タプルバリアント、または構造体バリアントを訪ねます。
            let value = visitor.visit_enum(Enum::new(self))?;
            // マッチングする閉じカッコを解析します。
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedEnum)
        }
    }

    // Serdeにおける識別子は、構造体のフィールドまたは列挙型のバリアントを識別する型です。
    // JSONに置いて、構造体のフィールドと列挙型のバリアントは文字列として表現されます。
    // 他のフォーマットにおいて、それらは数値インデックスとして表現されるかもしれません。
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // `deserialize_any`に似ていますが、データが無視されるため、`Deserializer`が、どの`Visitor`メソッドを呼び出しても
    // 違いがないことを示します。
    //
    // あるデシリアライザーは、`deserialize_any`よりもこれをより効率的に実装できます。
    // 例えば、一致する区切り文字を素早くスキップし、その間のデータに注意を払いません。
    //
    // あるフォーマットはこれを実装することができません。
    // `deserialize_any`と`deserialize_any`を実装できるフォーマットは、自己説明として知られます。
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

// JSONの配列またはマップをデシリアライズしているとき、コンマを正確に処理するために、最初の要素または最初の要素の
// 後の要素であるか追跡する必要があります。
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated {
            de,
            first: true,
        }
    }
}

// `SeqAccess`はシーケンスの要素を反復処理する能力を与えるために`Visitor`に提供されます。
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // 要素があるか確認します。
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        // コンマは最初の要素を除いて、すべての要素の前に要求されます。
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedArrayComma);
        }
        self.first = false;
        // 配列要素をデシリアライズします。
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess`は、マップのエントリを反復処理する能力を与えるために、`Visitor`に提供されます。
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // エントリがあるか確認します。
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // コンマは最初以外のすべてのエントリに要求されます。
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedMapComma);
        }
        self.first = false;
        // マップのキーをデシリアライズします。
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // コロンが`next_key_seed`の末尾または`next_value_seed`の最初で解析されたかどうかに違いはありません。
        // この場合、ここのコードは少し簡素です。
        if self.de.next_char()? != ':' {
            return Err(Error::ExpectedMapColon);
        }
        // マップの値をデシリアライズします。
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess`は、列挙型のどのバリアントにデシリアライズするかを決定するために、`Visitor`に提供されます。
//
// Serdeのすべての列挙型のデシリアライズメソッドが排他的に「外部タグ付け」列挙型表現を参照しないことに注意してｋづあさい。
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // `deserialize_enum`メソッドが`{`を解析したため、現在マップの内部にいます。
        // シードはマップのキーから自身をデシリアライズします。
        let val = seed.deserialize(&mut *self.de)?;
        // マップのキーから値を分離するコロンを解析します。
        if self.de.next_char()? == ':' {
            Ok((val, self))
        } else {
            Err(Error::ExpectedMapColon)
        }
    }
}

// `VariantAccess`は、デシリアライズするために決定された単独のバリアントの内容を確認する能力を与えるために
// `Visitor`に提供されます。　
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // もし、`Visitor`がこのバリアントをユニットバリアントであると予想した場合、入力は`deserialize_enum`内のケースで処理
    // される素朴な文字列であるべきです。
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // ニュータイプバリアントは、JSONで`{ NAME: VALUE }`で表現されるため、ここでは値にデシリアライズします。
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // タプルバリアントは、JSONで`{ NAME: [DATA...] }`で表現されるため、ここではデータのシーケンスにデシリアライズします。
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // 構造体バリアントは、JSONで`{ NAME: { K: V, ... } }`で表現されるため、ここでは内部マップにデシリアライズします。
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    let j = r#"{"int":1,"seq":["a","b"]}"#;
    let expected = Test {
        int: 1,
        seq: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Newtype":1}"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Tuple":[1,2]}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Struct":{"a":1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, from_str(j).unwrap());
}
```
