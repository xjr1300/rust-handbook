# デシリアライザーのライフタイムを理解する

<https://serde.rs/lifetimes.html>

[Deserialize](https://docs.rs/serde/1/serde/trait.Deserialize.html)と[Deserializer](https://docs.rs/serde/1/serde/trait.Deserializer.html)トレイトは、他のデシリアライゼーション関連のトレイトと同様に、両方とも`de`と呼ばれるライフタイムがあり、

```rust
trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D:Deserializer<'de>;
}
```

このライフタイムにより、Serdeは様々なデータフォーマットで効率的なゼロコピーデシリアライゼーションを安全に実行でき、Rust以外の他の言語では不可能または極めて不安全です。

```rust
#[derive(Deserialize)]
struct Users<'a> {
    id: u32,
    name: &'a str,
    screen_name: &'a str,
    location: &'a str,
}
```

ゼロコピーデシリアライゼーションは、文字列、文字列のバイトデータまたは入力保持するバイト配列を借用する上記`User`構造体のようなデータ構造にデシリアライズすることを意味します。
これは、それぞれここのフィールドの文字列を蓄積するためにメモリを割り当て、入力から新しく割り当てられたフィールドに文字列データをコピーすることを回避します。
Rustは、出力データ構造のスコープが終了するまで、入力データが長生きすることを保証し、出力データ構造が入力データを参照する間、入力データを失う結果となるダングリングポインタエラーを持つことが不可能であることを意味します。

## トレイト制約

`impl`ブロック、関数、またはその他の場所のいずれであっても、`Deserialize`トレイト制約を記述する主に2つの方法があります。

**`<'de, T> where T: Deserialize<'de>`**

これは、「`T`は**ある**ライフタイムからデシリアライズできる」ことを意味します。
呼び出し側は、それがどのライフタイムかを決定できます。
通常、例えば[serde_json::from_str](https://docs.rs/serde_json/1/serde_json/fn.from_str.html)のような関数のように、呼び出し側がデシリアライズするデータを提供するときに使用されます。
この場合、入力データはライフタイム`de`を持たなければならず、例えばそれは`&'de str`です。

**`<T> where T: DeserializeOwned`**

これは、「Tは**任意の**ライフタイムからデシリアライズできる」ことを意味します。
呼び出される側はライフタイムを決定できます。
普通、これは、デシリアライズされるデータが関数が戻る前に破棄されるため、`T`がそのデータから借用することができないようにしなくてはなりません。
例えば、base64でエンコードされたデータを入力として受け取り、それをbase64でデコードし、`T`型の値にデシリアライズし、そしてbase64でデコードされた結果を破棄する関数です。
この制約の他の一般的な使用は、[serde_json::from_reader](https://docs.rs/serde_json/1/serde_json/fn.from_reader.html)のような、IOストリームからデシリアライズする関数です。

より技術的に言えば、[DeserializeOwned](https://docs.rs/serde/1/serde/de/trait.DeserializeOwned.html)トレイトは、[高階トレイト制約（higher-rank trait bound）](https://doc.rust-lang.org/nomicon/hrtb.html)`for<'de> Deserializer<'de>`と同等です。
唯一の違いは、`DeserializeOwned`のほうが読みやすいことです。
これは、`T`がデシリアライズされるすべてのデータを所有することを意味します。

> 関数の引数で渡される通常のライフタイムは、あるインスタンスが存在するスコープをしめす。
> 高階トレイト制約は、**任意**のライフタイムを示し、特定のライフタイムに縛られない。
>
> ```rust
> // 任意のライフタイムを持つ文字列スライス参照を受け取る関数を引数として受け取る関数
> fn call_with_str<F>(f: F)
> where
>     F: for<'a> Fn(&'a str),
> {
>     f("hello");
>     f("world");
> }
>
> fn main() {
>     call_with_str(|s| println!("文字列: {}", s));
>
>     let s = String::from("固定文字列");
>     let f = |_: &str| println!("{}", s);
>     call_with_str(f);
> }
> // 文字列: hello
> // 文字列: world
> // 固定文字列
> // 固定文字列
> ```

`<T> where T: Deserialize<'static>`は決して欲しいものではないことに注意してください。
また`Deserialize<'de> +'static`も決して欲しいものではありません。
一般的に、`Deserialize`の近くのどこにでも`static`と記述することは、間違った道を進んでいる印です。
代わりに上記の制約の1つを使用してください。

## 一時的、借用、そして所有したデータ

Serdeデータモデルは、デシリアライズしているとき、3種類の文字列とバイト配列があります。
それらは[Visitor](https://docs.rs/serde/1/serde/de/trait.Visitor.html)トレイトの異なるメソッドに対応しています。

- **一時的**: `&str`を受け取る[visit_str](https://docs.rs/serde/1/serde/de/trait.Visitor.html#method.visit_str)
- **借用**: `&'de str`を受け取る[visit_borrowed_str](https://docs.rs/serde/1/serde/de/trait.Visitor.html#method.visit_borrowed_str)
- **所有**: `String`を受け取る[visit_string](https://docs.rs/serde/1/serde/de/trait.Visitor.html#method.visit_string)

一時的なデータは、それが渡されるメソッド呼び出しを超えて存続することが保証されません。
例えば[FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html)トレイトを使用してSerde文字列からIPアドレスみたいにデシリアライズするとき、それで十分です。
それが十分でないときは、[to_owned()](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html)呼び出しによって、そのデータがコピーされることです。
IOストリームからの入力が`Visitor`に渡される前にメモリ内にバッファーされるとき、またはエスケープシーケンスが処理されるために、結果として得られる文字列は入力中にそのまま（逐語的に）存在していないとき、デシリアライザーは一般的に一時的なデータを使用します。

借用されたデータは、`Deserializer`の`de`ライフタイムパラメーターよりも長生きすることが保証されます。
すべてのデシリアライザーが借用されたデータの処理をサポートしているわけではありません。
例えば、IOストリームからデシリアライズするとき、データを借用できません。

所有されたデータは、[Visitor](https://docs.rs/serde/1/serde/de/trait.Visitor.html)がそれに要求する限り生存することが保証されます。
あるビジターは所有したデータを受け取ることに利点があります。
例えば、Rustの`String`型の`Deserializer`実装は、すでにデシリアライズされたSerde文字列データの所有権を与えられるという利点があります。

## `Deserialize<'de>`ライフタイム

このライフタイムは、この型によって借用されたデータが有効でなければならない期間に関する制約を記録します。

この型によって借用されたデータのそれぞれのライフタイムは、その`Deserialize`実装の`de`ライフタイムで制約されなければなりません。
もし、この型がライフタイム`a`でデータを借用している場合、`de`は`a`よりも長生きするように制約されなければなりません。

```rust
struct S<'a, 'b, T> {
    a: &'a str,
    b: &'b str,
    bb: &'b str,
    t: T,
}

impl<'de: 'a + 'b, 'a, 'b, T> Deserialize<'de> for S<'a, 'b, T>
where
    T: Deserialize<'de>,
{
    /* ... */
}
```

もし、この型が`Deserializer`から何もデータを借用しない場合、`'de`ライフタイムに制約はありません。
そのような型は[DeserializedOwned](https://docs.rs/serde/1/serde/de/trait.DeserializeOwned.html)トレイトを自動的に実装します。

```rust
struct S {
    owned: String,
}

impl<'de> Deserialize<'de> for S {
    /* ... */
}
```

`'de`ライフタイムは、`Deserialize`実装に適用するために型内に現れるべきではありません。

```rust
- // これをしないでください。すぐ後で泣くことになります。
- impl<'de> Deserialize<'de> for Q<'de> {

+ // 代わりに次のようにしてください。
+ impl<'de: 'a, 'a> Deserialize<'de> for Q<'a> {
```

## `Deserializer<'de>`ライフタイム

次は、`Deserializer`から借用できるデータのライフタイムです。

```rust
struct MyDeserializer<'de> {
    input_data: &'de [u8],
    pos: usize,
}

impl<'de> Deserializer<'de> for MyDeserializer<'de> {
    /* ... */
}
```

もし、`Deserializer`が[visit_borrowed_str](https://docs.rs/serde/1/serde/de/trait.Visitor.html#method.visit_borrowed_str)または[visit_borrowed_bytes](https://docs.rs/serde/1/serde/de/trait.Visitor.html#method.visit_borrowed_bytes)を呼び出さない場合、`de`ライフタイムはライフタイムパラメーターを制約しません。

```rust
struct MyDeserializer<R> {
    read: R,
}

impl<'de, R> Deserializer<'de> for MyDeserializer<R>
where
    R: io::Read,
{
    /* ... */
}
```

## 導出実装内の借用データ

`&str`と`&[u8]`型のフィールドは、Serdeによって入力データから暗黙的に借用されます。
フィールドの他の型は、`#[serde(borrow)]`属性を使用することで、借用を選択できます。

```rust
use std::borrow::Cow;

use serde::Deserialize;

#[derive(Deserialize)]
struct Inner<'a, 'b> {
    // &strと&[u8]は暗黙的に借用されます。
    username: &'a str,

    // 他の型は明示的に借用されなくてはなりません。
    #[serde(borrow)]
    comment: Cow<'b, str>,
}

#[derive(Deserialize)]
struct Outer<'a, 'b, 'c> {
    owned: String,

    #[serde(borrow)]
    inner: Inner<'a, 'b>,

    // このフィールドは借用されません。
    not_borrowed: Cow<'c, str>,
}
```

この属性は、生成された`Deserialize`実装の`de`ライフタイムの制約を置くことで機能します。
例えば、上記で定義した`Outer`構造体の実装は次のように見えます。

```rust
// ライフタイム'aと'bは借用されますが、`cは借用されません。。
impl<'de: 'a + 'b, 'a, 'b, 'c> Deserializer<'de> for Outer<'a, 'b, 'c> {
    /* ... */
}
```

属性はどのライフタイムが借用されるべきかを明示的に指定するかもしれません。

```rust
use std::marker::PhantomData;

// この構造体は最初の2つのライフタイムを借用しますが、3番目はそうではありません。
#[derive(Deserialize)]
struct Three<'a, 'b, 'c> {
    a: &'a str,
    b: &'b str,
    c: PhantomData<&'c str>,
}

#[derive(Deserialize)]
struct Example<'a, 'b, 'c> {
    // 'aと'bのみを借用して、'cは借用しません。
    #[serde(borrow = "'a + 'b")]
    three: Tree<'a, 'b, 'c>,
}
```
