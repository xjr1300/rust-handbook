# 例

## JSONにおける構造体と列挙型

<https://serde.rs/json.html>

Serdeの`Serializer`は、フォーマット内でどのRustの構造体と列挙型が表現するための規則を選択する役割があります。
次は、[serde_json](https://github.com/serde-rs/json)データフォーマットによって選択された規則です。
一貫性を保つために、他の人間が読めるフォーマットは、可能であれば類似した規則を開発することが推奨されます。

```rust
struct W {
    a: i32,
    b: i32,
}
let w = W { a:0, b: 0};   // `{"a":0,"b":0}`で表現されます。

struct X(i32, i32);
let x = X(0, 0);    // `[0,0]`で表現されます。

struct Y(i32);
let y = Y(0); // 単に内部の値`0`で表現されます。

struct Z;
let z = Z;  // `null`で表現されます。

enum E {
    W { a: i32, b: i32 },
    X(i32, i32),
    Y(i32),
    Z,
}
let w = E::W { a: 0, b: 0}; // `{"W":{"a":0,"b":0}}`で表現されます。
let x = E::X(0, 0);         // `{"X":[0,0]}`で表現されます。
let y = E::Y(0);            // `{"Y":0}`で表現されます。
let z = E::Z;               // `"Z"`で表現されます。
```

## 列挙型の表現

次の列挙型を考えてください。

```rust
#[derive(Serialize, Deserialize)]
enum Message {
    Request { id: String, method: String, params: Params },
    Response { id: String, result: Value },
}
```

### 外部タグ付け

Serdeにおける列挙型のデフォルトの表現は、外部タグ付け列挙型表現と呼ばれます。
JSON構文で記述すると、次のようになります。

```json
{"Request": {"id": "...", "method": "...", "params": {...}}}
```

外部タグ付け表現は、バリアントの内容を解析し始める前に、扱っているバリアントを知ることができることで特徴づけられます。
この特徴は、広大な範囲のテキストとバイナリフォーマットで機能できるようにします。
`Serializer::serialize_*_variant`と`Deserializer:deserialize_enum`メソッドは、外部タグ付き表現を使用します

この表現は、任意の型のバリアントを処理できます。上記のような構造体バリアント、タプルバリアント、ニュータイプバリアント、そしてユニットバリアントです。

JSONと他の自己説明フォーマットに置いて、外部タグ付き表現は、可読性のために理想的でないことがよくあります。
Serdeは、3つの他に可能な方言を選択するための属性を提供しています。

すべての列挙型表現は`no-std`プロジェクトで機能しますが、`no-alloc`プロジェクトで機能するのは外部タグ付きは

### 内部タグ付け

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Request { id: String, method: String, params: Param },
    Response { id: String, result: Value },
}
```

JSON構文で内部タグ付き表現は次のように記述されます。

```json
{"type": "Request", "id": "...", "method": "...", "params": {...}}
```

扱っているバリアントを識別するタグは、コンテンツの内部にあり、次にバリアントの他のフィールドがあります。
この表現はJavaライブラリで一般的です。

この表現は、構造体バリアント、構造体またはマップを含むニュータイプバリアント、そしてユニットバリアントで機能しますが、タプルバリアントを含む列挙型で機能しません。
タプルバリアントを含む列挙型に`#[serde(tag = "...")]`属性を使用することはコンパイル時にエラーが発生します。

> ```rust
> // ユニットバリアント（データを持たない単純な形）
> enum Direction {
>     North,
>     South,
>     East,
>     West,
> }
>
> // タプルバリアント（名前のないフィールドを持つ）
> enum IpAddr {
>     V4(u8, u8, u8, u8),
>     V6(String),
> }
>
> // 構造体バリアント（名前付きフィールドを持つ）
> enum Message {
>     Quit,   // ユニットバリアント
>     Move { x: i32, y: i32 },    // 構造体バリアント
>     Write(String),  // タプルバリアント　
>     ChangeColor(i32, i32, i32),     // タプルバリアント
> }
>
> // ニュータイプバリアント　
> use std::collections::HashMap;
>
> struct User {
>     name: String,
>     age: u32,
> }
>
>
> enum Data {
>     UserInfo(User),    // 構造体を含むニュータイプバリアント
>     Metadata(HashMap<String, String>),  // マップを含むニュータイプバリアント
> }
> ```

### 隣接タグ付け

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Block {
    Para(Vec<Inline>),
    Str(String),
}
```

この表現は、Haskellの世界で一般的です。
JSON構文で次のように記述されます。

```json
{"t": "Para", "c": [{...}, {...}]}
{"t": "Str", "c": "the string"}
```

タグと内容は同じオブジェクトの2つのフィールドとしてそれぞれ隣接します。

隣接タグ付け列挙型をデシリアライズするために、`serde`クレートのCargoフィーチャーの`alloc`は有効にしなくてはなりません（デフォルトで有効です）。

### タグなし

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Request { id: String, method: String, params: Params },
    Response { id: String, result: Value },
}
```

JSON構文で、タグなし表現は次のように記述されます。

```json
{"id": "...", "method": "...", "params": {...}}
```

どのバリアントのデータを含んでいるかを明示的に識別するタグはありません。
Serdeは順番にそれぞれのバリアントに対してデータがマッチングするか思考して、デシリアライズに成功した最初の1つが返されます。

この表現は、バリアントの任意の型を含む列挙型を処理できます。

このタグなし列挙型の他の例として、次の列挙型は整数または2つの文字列の配列のどちらかからデシリアライズできます。

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Data {
    Integer(u64),
    Pair(String, String),
}
```

タグなし列挙型をデシリアライズするために、`serde`クレートのCargoフィーチャーの`alloc`は有効にしなくてはなりません（デフォルトで有効です）。

## フィールドのデフォルト値

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    // 「リソース」が入力内に含まれていない場合、デフォルトとして関数の結果を使用します。
    #[serde(default = "default_resource")]
    resource: String,

    // 「timeout」が入力に含まれていない場合、std::default::Defaultの型の実装を使用します。
    #[serde(default)]
    timeout: Timeout,

    // 「priority」が入力に含まれていない場合、デフォルトとして型のメソッドを使用します。
    // これはトレイトメソッドであるかもしれません。
    #[serde(default = "Priority::lowest")]
    priority: Priority,
}

fn default_resource() -> String {
    "/".to_string()
}

// 秒単位のタイムアウト
#[derive(Debug, Deserialize)]
struct Timeout(u32);

impl Default for Timeout {
    fn default() -> Self {
        Self(30)
    }
}

#[derive(Debug, Deserialize)]
enum Priority {
    ExtraHigh,
    High,
    Normal,
    Low,
    ExtraLow,
}

impl Priority {
    fn lowest() -> Self {
        Priority::ExtraLow
    }
}

fn main() {
    let json = r#"
        [
            {
                "resource": "/users"
            },
            {
                "timeout": 5,
                "priority": "High"
            }
        ]
    "#;

    let requests: Vec<Request> = serde_json::from_str(json).unwrap();

    // 最初のリクエストは、`resource="/users", timeout=30, priority=ExtraLow`を持ちます。
    println!("{:?}", requests[0]);

    // 2番目のリクエストは、`resource="/", timeout=5, priority=High`を持ちます。
}
```

## 構造体のフラット化

`flatten`属性は、フィールドのキーを親構造体にインライン化します。
`flatten`は、同じ構造体で複数回使用されるかもしれません。
それは名前を持つフィールドの構造体でのみサポートされており、それを適用するフィールドは構造体またはマップ型でなくてはなりません。

*注記*: `flatten`は[deny_unknown_fields](https://serde.rs/container-attrs.html#deny_unknown_fields)を使用する構造体内での組み合わせをサポートしていません。
外部または内部のフラット化した構造体のどちらかが、その属性を使用しないでください。

`flatten`属性は、次の2つの一般的なユースケースを提供します。

### 頻繁にグループ化されるキーを除外する

どれくらい多くの結果をリクエストされたか、全体の結果セットのどこまでを見ているのか、そして全体でどれくらい多くの結果が存在するのかを示すページネーションメタデータを含む結果ページを返す、ページネートされたAPIを考えてください。
もし、全体で1053件の結果を一度に100件ずつページングする場合、3番目のページは次のようになるでしょう。

```json
{
    "limit": 100,
    "offset": 200,
    "total": 1053,
    "users": [
        {"id": "49824073-979f-4814-be10-5ea416ee1c2f", "username": "john_doe"},
        ...
    ]
}
```

`"limit"`、`"offset"`そして`"total"`フィールドを持つ同じスキーマは、多くの異なるAPI問い合わせで共有されるかもしれません。
例えば、ユーザー、問題、プロジェクトなどを問い合わせするとき、ページネートされた結果を望むかもしれません。

この場合、一般的なページネーションメタデータフィールドを、それぞれのAPIレスポンスオブジェクトにフラット化できる共有された構造体に分解することが便利になります。

```rust
#[derive(Serialize, Deserialize)]
struct Pagination {
    limit: u64,
    offset: u64,
    total: u64,
}

#[derive(Serialize, Deserialize)]
struct Users {
    users: Vec<User>,
    #[serde(flatten)]
    pagination: Patination,
}
```

### 追加的なフィールドのキャプチャー

マップ型のフィールドは、構造体の他のすべてのフィールドによってキャプチャーされない追加的なデータを保持するためにフラット化できます。

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
struct User {
    id: String,
    username: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
```

例えば、フラット化された追加フィールドをキー`"mascot": "Ferris"`で埋める場合、次のJSON表現にシリアライズします。

```json
{
    "id": "49824073-979f-4814-be10-5ea416ee1c2f",
    "username": "john_doe",
    "mascot": "Ferris"
}
```

このデータのデシリアライゼーションは、フラット化された追加フィールドに`"mascot"`が戻されます。

## ジェネリック型境界の手動記述

ジェネリック型パラメーターを持つ構造体に対して`Serializer`と`Deserialize`を導出するとき、ほとんどSerdeはプログラマーからの助け無しで正確なトレイト制約を類推できます。
それは、正しい制約を推測するために経験則を使用しますが、最も重要なことは、それがシリアライズされたフィールドの一部であるすべての型パラメーター`T`に`T: Serialize`境界を、デシリアライズされたフィールドの一部であるすべての型パラメーター`T`に`T: Deserialize`境界を追加します。
経験則であるため、これは常に正しくないため、Serdeはプログラマーによって記述された境界で、自動的に生成された境界を置き換える脱出ハッチを提供しています。

```rust
use std::fmt::Display;
use std::str::FromStr;

use serde::{de, Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Outer<'a, S, T: 'a + ?Sized> {
    // Deserialize実装を導出するとき、Serdeはこのフィールドの型に`S: Deserialize`境界を生成する
    // ことを要求するかもしれません。
    // しかし、その`Deserializer`実装の代わりに、`deserialize_from_str`を解する事により、
    // 型の`FromStr`実装を使用するつもりであるため、`deserialize_from_str`を要求することで
    // 自動的に生成された境界を上書きします。
    #[serde(deserialize_with = "deserialize_from_str")]
    #[serde(bound(deserialize = "S: FromStr, S::Err: Display"))]
    s: S,

    // ここで、Serde`T: Deserialize`境界を生成することを望むかもしれません。
    // これは必要以上に厳しい条件です。
    // 実際に、下の`main`関数は`Deserialize`を実装していない`T=str`を使用します。
    // 自動的に生成された境界をより緩い境界で上書きします。
    #[serde(bound(deserialize = "Ptr<'a, T>: Deserialize<'de>"))]
    ptr: Ptr<'a, T>,
}

/// 文字列をデシリアライズすることで型`S`をデシリアライズして、結果を作成するために`S`の`FromStr`実装
/// を使用します。
/// ジェネリック型`S`は`Deserialize`の実装を要求しません。
fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

/// データを所有しているかもしれないし、しているかもしれない`T`へのポインターです。
/// デシリアライズするとき、常に所有するデータを作成します。
#[derive(Debug)]
enum Ptr<'a, T: 'a + ?Sized> {
    Ref(&'a T),
    Owned(Box<T>),
}

impl<'de, 'a, T: 'a + ?Sized> Deserialize<'de> for Ptr<'a, T>
where
    Box<T>: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Ptr::Owned)
}

fn main() {
    let j = r#"
        {
            "s": "1234567890",
            "ptr": "owned"
        }
    "#;

    let result: Outer<u64, str> = serde_json::from_str(j).unwrap();

    // result = Outer { s: 1234567890, ptr: Owned("owned") }
    println("result = {:?}", result);
}
```

## 独自のマップ型にデシリアライズを実装

```rust
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

// `Visitor`は、入力データに含まれる内容に依存して`Deserializer`が実行できるメソッドを保持する型です。
//
// マップの場合、正確に出力型を設定できるようにするために、型パラメーター`K`と`V`を必要としますが、状態を
// 要求しません。
// これは、Rustの「ゼロサイズ型」の例です。
// `PhantomData`は使用されていないジェネリックパラメーターについて、コンパイラーが不平を言わないようにします。
struct MyMapVisitor<K, V> {
    marker: PhantomData<fn() -> MyMap<K, V>>,
}

impl<K, V> MyMapVisitor<K, V> {
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

// これは、デシリアライザーが実行するトレイトです。
// 型がどのようにデシリアライズするかを知っている型であるデータのそれぞれの型に対して、ひとつのメソッドがあります。
// 例えば、整数から文字列からデシリアライズするような、ここに実装していない他に多くのメソッドがあります。
// デフォルトで、これらのメソッドはエラーを返し、整数または文字列からデシリアライズできないため、それは妥当です。
impl<'de, K, V> Visitor<'de> for MyMapVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    // Visitorが作成する型
    type Value = MyMap<K, V>;

    // このVisitorが受け取ることを予期するデータを述べるメッセージを書式化します。
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // デシリアライザーによって作成される抽象的な「マップ」から`MyMap`をデシリアライズします。
    // `MapAccess`の入力はマップ内のそれぞれのエントリを確認できるように、デシリアライザーによって
    // 作成されたコールバックです。
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = MyMap::with_capacity(access.size_hint().unwrap_or(0));

        // 入力に残っているエントリがある間、マップにそれらを追加します。
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

// これは、どのように`MyMap`をデシリアライズするかをSerde`に通知するトレイトです。
impl<'de, K, V> Deserialize<'de> for MyMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fne deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Visitorをインスタンス化し、`Deserializer`に入力データを処理するように要求し、
        // `MyMap`のインスタンスを生成します。
        deserializer.deserialize_map(MyMapVisitor::new())
    }
}
```

## Vecにバッファリングしないで値の配列を処理する

整数の配列があり、一度ですべてをメモリ内に配列全体を保持しないで、最大値を見つけたいケースを想像してください。
この手法は、データをデシリアライズした後ではなく、デシリアライズしている間に、データを処理する必要がある様々な状況に対応するために、適応できます。

```rust
use std::{cmp, fmt};
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use serde::de::{self, Visitor, SeqAccess};

#[derive(Deserialize)]
struct Outer {
    id: String,

    // 値のシーケンス（JSON配列）の最大値を計算することにより、このフィールドをデシリアライズします。
    #[serde(deserialize_with = "deserialize_max")]
    // 構造体のフィールドが`max_value`と名付けられているにも関わらず、`values`と呼ばれるJSON
    // フィールドから得られます。
    #[serde(rename(deserialize = "values"))]
    max_value: u64,
}

/// 値のシーケンスの最大値をデシリアライズします。
/// `Vec<T>`にデシリアライズしてから、アドで最大値を計算する場合のように、シーケンス全体がメモリに
/// バッファリングされません。
///
/// この関数は、`T`に対してジェネリックで、それは`Ord`を実装した任意の型になります。
/// 上記では、`T=u64`を使用しています。
fn deserialize_max<'de, T, D>(deserialize: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Ord,
    D: Deserialize<'de>,
{
    struct MaxVisitor<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for MaxVisitor<T>
    where
        T: Deserialize<'de> + Ord,
    {
        /// このVisitorの返却型です。
        /// このVisitorは型`T`の値のシーケンスの最大値を計算するため、最大値の型は`T`です。
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<T, S::Error>
        where
            S: SeqAccess<'de>,
        {
            // シーケンス内の最初の値を最大値として始めます。
            let mut max = seq.next_element()?.ok_or_else(||
                // 空のシーケンスの最大値を受け取ることはできません。
                de::Error::custom("no values in seq when looking for maximum")
            )?;

            // 追加的な値がある間、最大値を更新します。
            while let Some(value) = seq.next_element()? {
                max = cmp::max(max, value);
            }

            Ok(max)
        }
    }

    // ビジターを作成し、デシリアライザーにそれを駆動するように要求します。
    // デシリアライザーは、入力データ内にシーケンスが存在する場合、`visitor.visit_seq()`を呼び出します。
    let visitor = MaxVisitor(PhantomData);
    deserialize.deserialize_seq(visitor)
}

fn main() {
    let j = r#"
        {
            "id": "demo-deserialize-max",
            "values": [
                256,
                100,
                384,
                314,
                271
            ]
        }
    "#;

    let out: Outer = serde_json::from_str(j).unwrap();

    // 「max value: 384」を出力します。
    println!("max value: {}", out.max_value);
}
```

## 数値で列挙型をシリアライズ

[serde_repr](https://github.com/dtolnay/serde-repr)クレートは、同様の`Serialize`と`Deserialize`トレイトを導出する代替の導出マクロを提供しますが、C言語のような列挙型の表現に基づいて委譲します。
これは、JSONの文字列ではなく整数として書式されたC言語のような列挙型を許可します。
例えば・・・

```toml
# Cargo.toml
[dependencies]
serde = "1.0"
serde_json = "1.0"
serde_repr = "0.1"
```

```rust
use serde_repr::*;

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum SmallPrime {
    Two = 2,
    Three = 3,
    Five = 5,
    Seven = 7,
}

fn main() {
    use SmallPrime::*;

    let nums = vec![Two, Three, Five, Seven];

    // `[2,3,5,7]`を出力します。
    println!("{}", serde_json::to_string(&mums).unwrap());

    assert_eq!(Two, serde_json::from_str("2").unwrap());
}
```

## キャメルケースでフィールドをシリアライズ

```rust
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Person {
    first_name: String,
    last_name: String,
}

fn main() {
    let person = Person {
        first_name: "Graydon".to_string(),
        last_name: "Hoare".to_string(),
    };

    let json = serde_json::to_string_pretty(&person).unwrap();

    // 出力:
    //
    //    {
    //      "firstName": "Graydon",
    //      "lastName": "Hoare"
    //    }
    println!("{}", json);
}
```

## シリアライズでフィールドをスキップ

**注記**: `skip_serializing`はデシリアライズでフィールドをスキップしません。
もし、`skip_serializing`属性を追加しただけの場合、データのデシリアライズを試行すると、スキップしたフィールドのデシリアライズをまだ試行するため、失敗します。
シリアライズとデシリアライズ両方をスキップするために`skip`属性を使用してください（[フィールド属性: `skip`](https://serde.rs/field-attrs.html#skip)を参照してください）。
同様に、デシリアライズのみスキップするために`skip_deserializing`を使用してください。

```rust
use std::collections::BTreeMap as Map;

use serde::Serialize;

#[derive(Serialize)]
struct Resource {
    // 常にシリアライズします。
    name: String,

    // 決してシリアライズしません。
    #[serde(skip_serializing)]
    hash: String,

    // フィールドがスキップされるべきか決定するためにメソッドを使用します。
    #[serde(skip_serializing_if = "Map::is_empty")]
    metadata: Map<String, String>,
}

fn main() {
    let resources = vec![
        Resource {
            name: "Stack Overflow".to_string(),
            hash: "b6469c3f31653d281bbbfa6f94d60fea130abe38".to_string(),
            metadata: Map::new(),
        },
        Resource {
            name: "GitHub".to_string(),
            hash: "5cb7a0c47e53854cd00e1a968de5abce1c124601".to_string(),
            metadata: {
                let mut metadata = Map::new();
                metadata.insert("headquarters".to_string(),
                                "San Francisco".to_string());
                metadata
            },
        },
    ];

    let json = serde_json::to_string_pretty(&resources).unwrap();

    // 出力:
    //    [
    //      {
    //        "name": "Stack Overflow"
    //      },
    //      {
    //        "name": "GitHub",
    //        "metadata": {
    //          "headquarters": "San Francisco"
    //        }
    //      }
    //    ]
    println!("{}", json);
}
```

## 外部クレートの型のデシリアライズ／シリアライズを導出

Rustの[孤児ルール](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits)は、トレイトまたはトレイトを実装する型のどちらかが、`impl`で同じクレートで定義されている必要があるため、異なるクレートにある型に、直接`Serialize`と`Deserialize`を実装することはできません。

```rust
- use serde::Serialize;
- use other_crate::Duration;
-
- // Not allowed by orphan rule.
- impl Serialize for Duration {
-     /* ... */
- }
```

これを回避するために、Serdeは他のクレートの型に`Serialize`と`Deserialize`の実装を導出する方法を提供しています。
唯一の注意点は、Serdeの導出が処理するための型の定義を提供する必要があることです。
コンパイル時、Serdeは提供された定義内のすべてのフィールドが、外部の型のフィールドに一致するか確認します。

```rust
// これはモジュールではなく、誰かのクレートを模倣しています。
mod other_crate {
    // Serdeまたはクレートのどちらも、この構造体にSerializeとDeserializeを提供していません。
    pub struct Duration {
        pub secs: i64,
        pub nanos: i32,
    }
}

////////////////////////////////////////////////////////////////////////////////

use serde::{Serialize, Deserialize};
use other_crate::Duration;

// Serdeは外部の型のこの定義を呼び出します。
// それは外部のデータ構造の単なるコピーです。
// **外部**の属性はコードで導出することを意図した実際の型を渡します。
#[derive(Serialize, Deserialize)]
#[serde(remote = "Duration")]
struct DurationDef {
    secs: i64,
    nanos: i32,
}

// 現時点で、外部の型は自身の`Serialize`と`Deserialize`の実装をすべて持っているのとほとんど同じように
// 使用できます。
// `with`属性は外部の型の定義へのパスを与えます。
// フィールドの実際の型は外部の型で、定義した型でないことに注意してください。
#[derive(Serialize, Deserialize)]
struct Process {
    command_line: String,

    #[serde(with = "DurationDef")]
    wall_time: Duration,
}
```

もし、外部の型がすべてパブリックフィールドを持つ構造体または列挙型である場合、以上です。
もし、外部の型が、1つ以上のプライベートフィールドを保つ場合、ゲッターがプライベートフィールドように提供されていなければならず、変換は外部の型を構築するために提供されなくてはなりません。

```rust
// これは誰かのクレートを模倣しており、モジュールではありません。
mod other_crate {
    // Serdeと他のクレートは、この構造体に対して`Serialize`と`Deserialize`の実装を提供しません。
    pub struct Duration {
        secs: i64,
        nanos: i32,
    }

    impl Duration {
        pub fn new(secs: i64, nanos: i32) -> Self {
            Self {
                secs,
                nanos,
            }
        }

        pub fn seconds(&self) -> i64 {
            self.secs
        }

        pub fn subsec_nanos(&self) -> i32 {
            self.nanos
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

use serde::{Serialize, Deserialize};
use other_crate::Duration;

// 外部の構造体のすべてのプライベートフィールドのゲッターを提供します。
// ゲッターは`T`または`&T`のいずれかを返さなければならず、`T`はフィールドの型です。
#[derive(Serialize, Deserialize)]
#[serde(remote = "Duration")]
struct DurationDef {
    #[serde(getter = "Duration::seconds")]
    secs: i64,
    #[serde(getter = "Duration::subsec_nanos")]
    nanos: i32,
}

// 外部の型を構築するための変換を提供します。
impl From<DurationDef> for Duration {
    fn from(def: DurationDef) -> Self {
        Duration::new(def.secs, def.nanos)
    }
}

#[derive(Serialize, Deserialize)]
struct Process {
    command_line: String,

    #[serde(with = "DurationRef")]
    wall_time: Duration,
}
```

### 直接外部の実装を呼び出す

上記で紹介した通り、外部の実装は他の構造体のフィールドにある`#[serde(with = "...")]`を介して呼び出されることを意図されています。

外部実装を直接呼び出すことは、これがシリアライズされるまたはデシリアライズされる最上位の型である場合、言及した通り孤立ルールによてより複雑になります。
これらリモートの導出によって最終的に生成されるコードは、`Serialize`と`Deserialize`の実装ではありませんが、同じシグネチャーを持つ関連関数です。

```rust
// 技術的に、これは`Duration`に対する`Deserialize`実装、または`DurationDef`に対する
// `Deserialize`実装を生成しません。
//
// 代わりに、それは`Duration`型を返すデシリアライゼーションメソッド`DurationDef::deserialize`
// を生成します。
// このメソッドは、`Duration`用の`Deserialize`実装と同じシグネチャーを持ちますが、`Deserialize`
// 実装ではありません。
#[derive(Deserialized)]
#[serde(remote = "Duration")]
struct DurationDef {
    secs: i64,
    nanos: i32,
}
```

これを知っておくと、生成されたメソッドが`Deserialize`実装を渡すことにより、直接実行することができます。

```rust
let mut de = serde_json::Deserializer::from_str(j);
let dur = DurationDef::deserialize(&mut de)?;

// `dur`は`Duration`型です。
```

代わりに、外部の型をデシリアライズするプライベートなヘルパーとして、トップレベルのニュータイプラッパーを記述できます。

```rust
#[derive(Deserialize)]
struct Helper(#[serde(with = "DurationRef")] Duration);

let dur = serde_json::from_str(j).map(|Helper(dur) | dur)?;

// `dur`は`Duration`型です。
```

## 構造体に手作業で`Deserialize`を実装する

[derive](https://serde.rs/derive.html)が仕事を完了できないときのみ。

`Deserialize`の実装は、次の構造体に従って対応します。

```rust
struct Duration {
    secs: u64,
    nanos: u32,
}
```

構造体のデシリアライズは、フィールド名を保持する文字列を割り当てることを避けるために、[マップをデシリアライズ](https://serde.rs/deserialize-map.html)するよりも、多少より複雑になります。
代わりに、`&str`からデシリアライズされる`Field`列挙型があります。

実装は、構造体が、PostcardのようなシーケンスまたはJSONのようなマップのように、データフォーマットで表現する2つの方法をサポートします。

```rust
use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Secs, Nanos }

        // この部分は独立して生成することもできます。
        //
        //  #[derive(Deserialize)]
        //  #[serde(field_identifier, rename_all = "lowercase")]
        // enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write+str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "secs" => Ok(Field::Secs),
                            "nanos" => Ok(Field::Nanos),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration");
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
            where
                V: SeqAccess<'de>
            {
                let secs = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let nanos = seq.next_element()?;
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Duration::new(secs, nanos))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Duration, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut secs = None;
                let mut nanos = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Secs => {
                            if secs.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            secs = Some(map.next_value()?);
                        }
                        Field::Nanos => {
                            if nanos.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            nanos = Some(map.next_value()?);
                        }
                    }
                }
                let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
                let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
                Ok(Duration::new(secs, nanos))
            }
        }

        const FIELDS: &[&str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
    }
}
```

## データの破棄

[IgnoreAny](https://docs.rs/serde/1/serde/de/struct.IgnoredAny.html)型はデシリアライザーからデータを破棄する効率的な方法を提供します。

これを、デシリアライズしたデータについての情報を何も保存せずに、任意の型からデシリアライズできる`serde_json::Value`のように考えてください。

```rust
use std::fmt;
use std::marker::PhantomData;

use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, Visitor, SeqAccess,
    IgnoredAny,
};
use serde_json::json;

// シードは、インデックス`n`の前後の任意の型の要素を効率的に破棄する一方で、シーケンスの`n`番目の要素
// のみをデシリアライズするために使用できます。　
//
// 例えば、インデックス3の要素のみをデシリアライズします。
//
//  NthElement::new(3).deserialize(deserializer)
pub struct NthElement<T> {
    n: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T> NthElement<T> {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for NthElement<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence in which we care about element {}", self.n)
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        // 最初の`n`個の要素をスキップ
        for i in 0..self.n {
            // `n`番目の要素を得る前にシーケンスが終了した場合はエラーです。
            if seq.next_element::<IgnoreAny>()?.is_none() {
                return Err(de::Error::invalid_length(i, &self));
            }
        }

        // 関心のある要素をデシリアライズします。
        let nth = seq.next_element()?
            .ok_or_else(|| de::Error::invalid_length(self.n, &self))?;

        // `n`番目より後のシーケンス内の残りの要素をスキップ
        while let Some(IgnoreAny) = seq.next_element()? {
            // 無視します。
        }

        Ok(nth)
    }
}

impl<'de, T> DeserializeSeed<'de> for NthElement<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

fn main() {
    let array = json!(["a", "b", "c", "d", "e"]);

    let nth: String = NthElement::new(3).deserialize(&array).unwrap();

    println!("{nth}");
    assert_eq!(nth, array[3]);
}
```

## フォーマットを他のフォーマットに変換する

[serde-transcode](https://github.com/sfackler/serde-transcode)クレートは、メモリ内に入力全体の中間形式を集めることなしで、任意のSerdeの`Deserializer`を任意のSerdeの`Serializer`に変換する機能を提供します。
これは、メモリ効率が高いストリーミング方式で、任意の自己説明Serdeデータフォーマットを、他のSerdeデータフォーマットに変換する完全に一般的な方法を提供します。

例えば、JSONデータのストリームをCBORデータのストリームに変換したり、フォーマットされていないJSONを整頓された形式に変換できます。

次の例は、ストリーミング方法でJSON文字列から不要なホワイトスペースを除去するGoの[json.Compact](https://golang.org/pkg/encoding/json/#Compact)と同等の実装です。

```rust
use std::io;

fn main() {
    // 多くのホワイトスペースを持つJSONの入力です。
    let input = r#"
        {
            "a boolean": true,
            "an array": [3, 2, 1]
        }
    "#;

    // JSONデシリアライザーです。ここで任意のSerdeデシリアライザーを使用できます。
    let mut deserializer = serde_json::Deserializer::from_str(input);

    // 圧縮されたJSONシリアライザーです。ここで任意のSerdeのシリアライザーを使用できます。
    let mut serializer = serde_json::Serializer::new(io::stdout());

    // 標準出力に`{"a boolean":true,"an array":[3,2,1]}`を出力します。
    // この行は、任意の自己説明デシリアライザーとシリアライザーといっしょに機能します。
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
}
```

## 文字列または構造体のどちらかをデシリアライズする

[docker-compose.yml](https://docs.docker.com/compose/compose-file/#/build)ファイルは、文字列または構造体のどちらかになる"build"キーを持ちます。

```yaml
build: ./dir

# --- or ---

build:
  context: ./dir
  dockerfile: Dockerfile-alternate
  args:
    buildno: 1
```

その構成ファイルは他の場所でも同じパターンを使用して、通常、前に存在した文字列フィールドは、より複雑なデータを処理するために拡張されています。

一般的な方法として、Rustの[FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html)トレイトと、Serdeの`deserialize_with`属性を使用して、このパターンを処理できます。

```rust
use std::collections::BTreeMap as Map;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use void::Void;

fn main() {
    let build_string = "
        build: ./dir
    ";
    let service: Service = serde_yaml::from_str(build_string).unwrap();

    // context="./dir"
    // dockerfile=None
    // args={}
    println!("{service:?}");

    let build_struct = "
        build:
            context: ./dir
            dockerfile: Dockerfile-alternate
            args:
                buildno: '1'
    ";
    let service: Service = serde_yaml::from_str(build_struct).unwrap();

    // context="./dir"
    // dockerfile=Some("Dockerfile-alternate")
    // args={"buildno": "1"}
    println!("{:?}", service);
}

#[derive(Debug, Deserialize)]
struct Service {
    // `string_or_struct`関数は、文字列が与えられた場合、型の`FromStr`実装に、構造体が
    // 与えられた場合に型の`Deserialize`実装にデシリアライザーションを委譲します。
    // 関数は、フィールド型`T`に対してジェネリックなため（ここで`T`は｀Build`です）、
    // `FromStr`と`Deserialize`ｎ両方を実装しているフィールドを拒絶できます。
    #[serde(deserialize_with = "string_or_struct")]
    build: Build,
}

#[derive(Debug, Deserialize)]
struct Build {
    // これは唯一要求されるフィールドです。
    context: String,

    dockerfile: Option<String>,

    // 入力に`args`が存在しないとき、この属性は、この場合はからのマップである、`Default::default()`
    // を使用することをSerdeに伝えます。
    // `#[serde(default)]`についての詳細は「フィールドのデフォルト値」の例を参照してください。
    #[serde(default)]
    args: Map<String, String>,
}

// `string_or_struct`関数は、もし入力ファイルが文字列を含み、構造体を含んでいない場合、
// この実装を使用して、`Build`をインスタンス化します。
// `docker-compose.yml`のドキュメントによると、文字列それ自身は単に`context`フィールドのみが設定
// された`Build`を表現します。
//
// > `build`はビルドコンテキストのパスを含む文字列か、またはコンテキストを記述したパスとオプションの
// > `dockerfile`と`arg`を持つオブジェクトのどちらかを記述できます。
impl FromStr for Build {
    // この`from_str`の実装は決して失敗しないため、エラー型として不可能な`Void`型を使用します。
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Build {
            context: s.to_string(),
            dockerfile: None,
            args: Map::new(),
        })
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserializer<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // これは、`T`の`FromStr`実装に文字列型を渡して、`T`の`Deserialize`実装にマップ型を渡す
    // `Visitor`です。
    // `PhantomData`は`T`が使用されていないジェネリックパラメーターとしてコンパイラが不平を言わない
    // ようにします。
    // `T`は`Visitor`実装に対して`Value`の型を知らせるために必要です。
    struct StringOrStruct<T>(PhantomData<fn() -> T);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserializer<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer`は、`MapAccess`を`Deserializer`に変換する
            // ラッパーで、`T`の`Deserialize`実装への入力として使用できるようにします。
            // `T`はマップビジターからのエントリを使用してそれ自身をデシリアライズします。
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
```

## エラー型の変換

ある状況において、あるフォーマットの値は他のフォーマット内のデータ内に含まれていなければなりません。
例えば、[TerraformのIAMポリシー](https://www.terraform.io/docs/providers/aws/r/iam_policy.html)は、HCL構成の中に含まれたJSON文字列として表現されています。

単に文字列として内部の値を扱うことは単純ですが、もし、内部と外部の値の両方を操作する場合、一度にそれらすべてをシリアライズまたはデシリアライズすることが時々便利になります。

そのような状況で時々発生する障害は、エラーを正しく処理することです。
2つのフォーマットは（おそらく）異なるエラー型があるため、なんらかの変換が必要です。

この例は、簡素なIAMポリシーを含んだ簡素なHCLリソースです。
そのポリシードキュメントは、シリアライズされるときJSON文字列として表現されます。

```rust
use serde::{Serialize, Deserialize};

#[derive(Serializer, Deserialize)]
struct Resource {
    name: String,

    #[serde(with = "as_json_string")]
    policy: Policy,
}

#[derive(Serialize, Deserialize)]
struct Policy {
    effect: String,
    action: String,
    resource: String,
}

// JSON文字列としてネストした値を扱うシリアライズとデシリアライズのロジックです。
mod as_json_string {
    use serde_json;
    use serde::ser::{Serialize, Serializer};
    use serde::de::{Deserialize, DeserializeOwned, Deserializer};

    // JSON文字列にシリアライズし、出力フォーマットに文字列をシリアライズします。
    pub fn serialize<T, S>(valuer: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serialize,
    {
        use serde::ser::Error;
        let j = serde_json::to_string(value).map_err(Error::custom)?;
        j.serialize(serializer)
    }

    // 入力フォーマットから文字列をデシリアライズして、JSONとして文字列の内容をデシリアライズします。
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let j = String::deserialize(deserializer)?;
        serde_json::from_str(&j).map_err(Error::custom)
    }
}

fn main() {
    let resource = Resource {
        name: "test_policy".to_owned(),
        policy: Policy {
            effect: "Allow".to_owned(),
            action: "s3:ListBucket".to_owned(),
            resource: "arn:aws:s3:::example_bucket".to_owned(),
        },
    };

    let y = serde_yaml::to_string(&resource).unwrap();
    println!("{y}");
}
```

## 日付のカスタムフォーマット

これは、[chrono](https://github.com/chronotope/chrono)トレイトを使用して、カスタム日付フォーマットで含んでいるJSONデータをシリアライズまたはデシリアライズします。
`with`属性はカスタム表現を処理するためのロジックを提供するために使用されています。

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StructWithCustomDate {
    // `DataTime`はSerdeで標準でサポートされていますが、RFC3339フォーマットを使用します。
    // 望むフォーマットを使用できるようにカスタムロジックを提供します。
    #[serde(with = "my_date_format")]
    pub timestamp: DateTime<Utc>,

    // 構造体の他のフィールド　
    pub bidder: String,
}

mod my_date_format {
    use chrono::{DateTime, Utc, NativeDateTime};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // `serialize_with`関数のシグネチャーは次のパターンに従わなくてはなりません。
    //
    //      fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //      where
    //          S: Serializer,
    //
    // それでも、入力型`T`に対してジェネリックであるかもしれません。
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // `deserialize_with`関数のシグネチャーは次のパターンに従わなければなりません。
    //
    //      fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //      where
    //          D: Deserializer<'de>,
    //
    // それでも、出力　型`T`に対してジェネリックであるかもしれません。
    pub fn deserialize<'de, D>(
        deserializer: D
    ) ->Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserialize)?;
        lt dt = NativeDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_native_utc_and_offset(dt, Utc))
    }
}

fn main() {
    let json_str = r#"
        {
            "timestamp": "2017-02-16 21:54:30",
            "bidder": "Skrillex",
        }
    "#;

    let data: StructWithCustomDate = serde_json::from_str(json_str).unwrap();
    println!("{date:#?}");

    let serialized = serde_json::to_string_pretty(&data).unwrap();
    println!("{serialized}");
}
```
