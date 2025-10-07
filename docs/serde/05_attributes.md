# 属性

<https://serde.rs/attributes.html>

[属性](https://doc.rust-lang.org/book/attributes.html)は、Serdeの導出によって生成された`Serialize`と`Deserialize`実装をカスタマイズするために使用されます。
それらは、Rustコンパイラーバージョン1.15以上を要求します。

属性の3つのカテゴリーがあります。

- [コンテナ属性](https://serde.rs/container-attrs.html) - 構造体または列挙型定義に適用
- [バリアント属性](https://serde.rs/variant-attrs.html) - 列挙型のバリアントに適用
- [フィールド属性](https://serde.rs/field-attrs.html) - 構造体のフィールドまたは列挙型内のバリアント1つに適用

```rust
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]   // <-- これはコンテナ属性
struct S {
    #[serde(default)] // <-- これはフィールド属性
    f: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "e")]  // <-- これもコンテナ属性>
enum E {
    #[serde(rename = "a")]  // <-- これはバリアント属性
    A(String),
}
```

単独の構造体、列挙型、バリアント、またはフィールドは、それに複数の属性を持つかもしれないことに注意してください。

## コンテナ属性

### `#[serde(rename = "name")]`

構造体または列挙型のRustの名前の代わりに、与えられた名前でシリアライズとデシリアライズをします。

シリアライゼーションとデシリアライゼーションで異なる名前を指定できます。

- `#[serde(rename(serialize = "ser_name"))]`
- `#[serde(rename(deserialize = "de_name"))]`
- `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`

### `#[serde(rename_all = "...")]`

すべてのフィールド（構造体の場合）またはバリアント（列挙型の場合）を与えられたケース規則で名前を変更します。
可能な値は、`"lowercase"`、`"UPPERCASE"`、`"PascalCase"`、`"camelCase"`、`"snake_case"`、`"SCREAMING_SNAKE_CASE"`、`"kebab-case"`、`"SCREAMING-KEBAB-CASE"`です。

シリアライゼーションとデシリアライゼーションで異なるケースを指定できます。

- `#[serde(rename_all(serialize = "..."))]`
- `#[serde(rename_all(deserialize = "..."))]`
- `#[serde(rename_all(serialize = "...", deserialize = "..."))]`

### `#[serde(rename_all_fields = "...")]`

列挙型のすべての構造体のバリアントに与えられたケース規則で`rename_all`を適用します。
可能な値は、`"lowercase"`、`"UPPERCASE"`、`"PascalCase"`、`"camelCase"`、`"snake_case"`、`"SCREAMING_SNAKE_CASE"`、`"kebab-case"`、`"SCREAMING-KEBAB-CASE"`です。

シリアライゼーションとデシリアライゼーションで異なるケースを指定できます。

- `#[serde(rename_all_fields(serialize = "...")]`
- `#[serde(rename_all_fields(deserialize = "...")]`
- `#[serde(rename_all_fields(serialize = "...", deserialize = "...")]`

### `#[serde(deny_unknown_fields)]`

デシリアライズしているときに、不明なフィールドに出会ったときに常にエラーになります。
この属性が存在しないとき、JSONのような自己記述型フォーマットで、不明なフィールドは無視されます。

*注意*: 外側の構造チアでもフラット化されたフィールドでも、この属性は[flatten](https://serde.rs/field-attrs.html#flatten)との組み合わせをサポートしていません。

### `#[serde(tag = "type")]`

列挙型において、与えられたタグで、内部的にタグ付けした列挙型表現を使用します。
この表現の詳細は[列挙型の表現](https://serde.rs/enum-representations.html)を参照してください。

名前付きフィールドを持つ構造体において、構造体の名前（または`serde(rename)`の値）を、構造体の実際のすべてのフィールドの前にある、指定されたキーを持つフィールドとしてシリアライズします。

> ```rust
> #[serde(tag = "type")]
> enum Message {
>     Text { content: String },
>     Image { url: String },
> }
> ```
>
> 上記の場合、シリアライズ結果は次のようになる。
>
> ```text
> { "type": "Text", "content": "Hello" }
> ```
>
> ```rust
> #[serde(tag = "type")]
> struct Point {
>     x: i32,
>     y: i32,
> }
> ```
>
> 上記の場合、シリアライズ結果は次のようになる。
>
> ```text
> { "type": "Point", "x": 1, "y": 2 }
> ```

### `#[serde(tag = "t", content = "c")]`

この列挙型には、隣接タグ付きの列挙型表現を使用します。
タグとコンテンツには、指定されたフィールド名を使用します。
この表現の詳細については、[列挙型表現](https://serde.rs/enum-representations.html)を参照してください。

### `#[serde(untagged)]`

列挙型にタグのない列挙型表現を使用します。
この表現の詳細については、[列挙型表現](https://serde.rs/enum-representations.html)を参照してください。

どのバリアントも一致しないとき、`untagged`は情報エラーを生成しませんが、これは[expecting](https://serde.rs/container-attrs.html#expecting)メッセージを追加することで改善できます。

パフォーマンスが重視されるコードでは、`luntagged`による実装方法はコストが高くなる可能性があります。
`Deserialize`トレイトを手動で実装するか、[serde-untagged](https://docs.rs/serde-untagged)クレートを利用することを検討してください。

### `#[serde(bound = "T: MyTrait")]`

`Serialize`と`Deserialize`実装用の`Where`句です。
これはSerdeによって推測されたトレイト制約を置き換えます。

シリアライズとデシリアライズで異なる制約を指定できます。

- `#[serde(bound(serialize = "T: MySerTrait"))]`
- `#[serde(bound(deserialize = "T: MyDeTrait"))]`
- `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]`

### `#[serde(default)]`

デシリアライズしているとき、不足しているフィールドが構造体の`Default`実装で満たされます。
構造体のみ許可されます。

### `#[serde(default = "path")]`

デシリアライズしているとき、不足しているフィールドが、与えられた関数またはメソッドによって返されたメソッドで満たされます。
関数は`fn() ->`として呼び出し可能でなくてはなりません。
例えば、`default = "my_default"`は`my_default()`を呼び出し、`default = "SomeTrait::some_default"`は`SomeTrait::some_default()`を呼び出します。
構造体のみ許可されます。

### `#[serde(remote = "...")]`

これは、[外部の型](https://serde.rs/remote-derive.html)の`Serialize`と`Deserialize`を導出するために使用されます。

### `#[serde(transparent)]`

正確に1つのフィールドを持つニュータイプ構造体または波括弧構造体を、その1つのフィールドが単独でシリアライズまたはデシリアライズされる場合と同じ用に、シリアライズまたはデシリアライズします。

### `#[serde(from = "FromType")]`

`FromType`にデシリアライズすることで、この型をデシリアライズした後、変換します。
この型は`From<FromType>`を実装しなければならず、`FromType`は`Deserialize`を実装していなければなりません。

### `#[serde(try_from = "FromType")]`

`FromType`にデシリアライズすることで、この型をデシリアライズした後、失敗する可能性のある変換をします。
この型は`Display`を実装したエラー型で`TryFrom<FromType>`を実装しなければならず、`FromType`は`Deserialize`を実装していなければなりません。

### `#[serde(into = "IntoType")]`

指定された`IntoType`に変換することで、この型をシリアライズして、それをシリアライズします。
この型は、`Clone`と`Into<IntoType>`を実装していなければならず、`IntoType`は`Serialize`を実装していなければなりません。

### `#[serde(crate = "...")]`

生成されたコードからSerde APIを参照するときに使用する`serde`クレートインスタンスのパスを指定します。
これは、通常、別のクレート内の公開マクロから導出した`Serde`を最エクスポートして呼び出す場合にのみ適用されます。

### `#[serde(expecting = "...")]`

デシリアライゼーションエラーメッセージ用に、カスタムタイプの期待するテキストを指定します。
これは、`Visitor`コンテナで生成された`expecting`メソッドによって使用され、タグのない列挙型のフォールスルーメッセージとして使用されます。

### `#[serde(variant_identifier)]`

データ形式が外部タグ付き列挙型バリアントタグの表現として使用している文字列または整数のいずれかをデシリアライズします。
一般的に、人間が読める形式ではバリアントは文字列名で表現され、コンパクトなバイナリ形式では整数インデックスで表現されます。
この属性は、すべてのユニットバリアントを含む列挙型にのみ適用できます。

### `#[serde(field_identifier)]`

データ形式が構造体フィールド識別子の表現としてどちらを使用するかに応じて、文字列または整数のいずれかをデシリアライズします。
この属性は、最終的なバリアントがニュータイプバリアントになることが許可されている点で[variant_identifier](https://serde.rs/container-attrs.html#variant_identifier)と異なります。
これは、serde[other](https://serde.rs/variant-attrs.html#other)と同様に、入力がこの列挙型のユニットバリアントのいずれにも一致しない場合にデシリアライズされます。

## バリアント属性

### `#[serde(rename = "name")]`

このバリアントのRustの名前の代わりに与えられた名前でシリアライズまたはデシリアライズします。

シリアライゼーションとデシリアライゼーションで異なる名前を指定できます。

- `#[serde(rename(serialize = "ser_name"))]`
- `#[serde(rename(deserialize = "de_name"))]`
- `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`

### `#[serde(alias = "name")]`

与えられた名前またはそのRustの名前からこのバリアントをデシリアライズします。
同じバリアントに複数の可能性のある名前を指定するために繰り返されます。

### `#[serde(rename_all = "...")]`

構造体のすべてのフィールドを与えられたケース規則で名前を変更します。
可能な値は、`"lowercase"`、`"UPPERCASE"`、`"PascalCase"`、`"camelCase"`、`"snake_case"`、`"SCREAMING_SNAKE_CASE"`、`"kebab-case"`、`"SCREAMING-KEBAB-CASE"`です。

シリアライゼーションとデシリアライゼーションで異なるケースを指定できます。

- `#[serde(rename_all(serialize = "..."))]`
- `#[serde(rename_all(deserialize = "..."))]`
- `#[serde(rename_all(serialize = "...", deserialize = "..."))]`

### `#[serde(skip)]`

このバリアントをシリアライズまたはデシリアライズしません。

### `#[serde(skip_serializing)]`

このバリアントをシリアライズしません。
このバリアントをシリアライズすることを試みることは、エラーとして扱われます。

### `#[serde(skip_deserializing)]`

このバリアントをデシリアライズしません。

### `#[serde(serialize_with = "path")]`

その`Serialize`実装とは異なる関数を使用して、このバリアントをシリアライズします。
与えられた関数は、`fn<S>(&FIELD0, $FIELD1, ..., S) -> Result<S::Ok, S::Error> where S: Serializer`として呼び出し可能でなくてはなりません。
ただし、`FIELD{n}`型全体にわたって汎用的に使用することもできます。
`serialize_with`を使用されたバリアントは、`Serialize`を導出できることを要求されません。

`Field{n}`はバリアントのすべてのフィールドに存在します。
したがって、ユニットバリアントは単に`S`引数を持ち、タプリまたは構造体のバリアントはすべてのフィールドの引数があります。

### `#[serde(deserialize_with = "path")]`

その`Deserialize`の実装とは異なる関数を使用して、このバリアントをデシリアライズします。
与えられた関数は`fn<'de, D>(D) -> Result<FIELDS, D::Error> where D:Deserializer<'de>`として呼び出されなければなりませんが、`FIELDS`の要素がジェネリックであるかもしれません。
`deserialize_with`を使用したバリアントは、`Deserialize`を導出できることを要求されません。

`FIELDS`はバリアントのすべてのフィールドのタプルです。
ユニットバリアントは、`FIELDS`型として`()`になります。

### `#[serde(with = "module")]`

`serialize_with`と`deserialize_with`の組み合わせです。
Serdeは`serialize_with`間s縫うとして`$module::serialize`を、`deserialize_with`関数として`$module::deserialize`を使用します。

### `#[serde(bound = "T: MyTrait")]`

`Serialize`と`Deserialize`実装の`wh`ere`句です。
これは、このバリアントに対して、任意のトレイト制約をSerdeによって予想されたものに置き換えます。

シリアライゼーションとデシリアライゼーションで異なる制約を指定できます。

- `#[serde(bound(serialize = "T: MySerTrait"))]`
- `#[serde(bound(deserialize = "T: MyDeTrait"))]`
- `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T:MyTrait"))]`

### `#[serde(borrow)]`と`E[serde(borrow = "'a +'b +...")]`

ゼロコピーデシリアライゼーションを使用することで、デシリアライザーからこのフィールド用のデータを借用します。
[この例](https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl)を確認してください。

### `#[serde(other)]`

もし、列挙型のタグが、この列挙型内の他のバリアントのタグと異なる場合、このバリアントをデシリアライズします。
内部的にタグ付けされた列挙型または隣接してタグ付けされた列挙型内のユニットバリアントでのみ許可されます。

例えば、ヴァリアント`A`、`B`と、`serde(other)`でマークされた`Unknown`を含んでいる`serde(tag = "variant")`で内部的にタグ付けられた列挙体があり、`Unknown`バリアントは、入力の`"variant"`フィールドが`"A"`または`"B"`でない場合は、常に`Unknown`バリアントがデシリアライズされます。

### `#[serde(untagged)]`

[列挙型表現](https://serde.rs/enum-representations.html)に関係なく、タグのないバリアントとして、つまりバリアント名を記録しないバリアントのデータとしてシリアライズまたはデシリアライズします。

タグのないバリアントは、列挙型定義の最後に並び替えられなければなりません。

### 追記: 列挙型のシリアライズとデシリアライズ

#### 外部タグ付き（externally tagged）

列挙型はデフォルトで外部タグ付きでシリアライズまたはデシリアライズされる。

```rust
#[derive(Serialize, Deserialize)]
enum Message {
    Text { content: String },
    Ping,
}

let message = Message::Text { content: "hello".to_string() };
```

上記`message`をJSON形式でシリアライズしたとき、キーはバリアント名、値はオブジェクトとなる。

```json
{ "Text": { "content": "hello" } }
```

#### 内部タグ付き（internally tagged）

```rust
#[derive(Serialize, Deserialize)]
enum Message {
    Text { content: String },
    Ping,
}

let message = Message::Text { content: "hello".to_string() };
```

上記`message`をJSON形式でシリアライズしたとき、`type`とフィールド名`content`をキーに持つオブジェクトとなる。

```json
{ "type": "Text", "content": "hello" }
```

#### 隣接タグ付き（adjacently tagged）

- `type`（タグ）と `data`（中身）を分けて格納
- 内部タグ付きと違い、**任意の形（タプル型・ユニット型・構造体型）**のバリアントを扱える
- ただし入れ子が1段増える

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Message {
    Text { content: String },
    Ping,
}
```

```json
{ "type": "Text", "data": { "content": "hello" } }
```

#### 非タグ付き（untagged）

- タグ情報を一切出力しない
- データの「形（フィールド構造）」だけでどのバリアントかを推測
- フィールド構造が同じ場合、デシリアライズ時に失敗する可能性あり

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Text { content: String },
    Ping,
}
```

```json
{ "content": "hello" }
```

## フィールド属性

### `#[serde(rename = "name")]`

Rustの名前の代わりに与えられた名前でこのフィールドをシリアライズまたはデシリアライズします。
これは、[キャメルケースでフィールドをシリアライズ](https://serde.rs/attr-rename.html)する場合は、予約されたRustキーワードを名前に持つフィールドをシリアライズすつ場合に便利です。

シリアライズとデシリアライズで異なる名前を指定できます。

- `#[serde(rename(serialize = "ser_name"))]`
- `#[serde(rename(deserialize = "de_name"))]`
- `#[serde(rename(serialize = "ser_name", deserialize = "de_name"))]`

### `#[serde(alias = "name")]`

与えられた名前またはそのRustの名前でこのフィールドをデシリアライズします。
同じフィールドに対して複数の可能性がある名前を指定するときに繰り返されます。

### `#[serde(default)]`

デシリアライズするときに値が存在しない場合、`Default::default()`を使用します。

### `#[serde(default = "path")]`

デシリアライズするときに値が存在しない場合、デフォルト値を得るために関数を呼び出します。
与えられった関数は、`fn() -> T`で呼び出せなければなりません。
例えば、`default = "empty_value"`は`empty_value()`を呼び出し、`default = "SomeTrait::some_default"`は`SomeTrait::some_default()`を呼び出します。

### `#[serde(flatten)]`

このフィールドの内容を、定義されているコンテナにフラット化します。

これは、シリアライズ表現とRustデータ構造表現の間の階層を1つ取り除きます。
これは、共通のキーを共有された構造体に分割したり、余ったフィールドを任意の文字列キーでマップに入れたりするときに便利です。
[構造体のフラット化](https://serde.rs/attr-flatten.html)ページは例をいくつか提供しています。

*注記*: この属性は、`deny_unknown_fields`を使用した構造体での組み合わせをサポートしていません。
外側または内側のフラット化された構造体の土地らかがこの属性を使用するべきです。

### `#[serde(skip)]`

このフィールドをスキップして、シリアライズまたはデシリアライズしません。

デシリアライズしているとき、Serdeはこのフィールドのデフォルト値を得るために、`Default::default()`または`default = "..."`で与えられた関数を使用します。

### `#[serde(skip_serializing)]`

シリアライズしているとき、このフィールドをスキップしますが、デシリアライズしているときはスキップしません。

### `#[serde(skip_deserialization)]`

デシリアライズしているとき、このフィールドをスキップしますが、シリアライズしているときはスキップしません。

デシリアライズしているとき、Serdeはこのフィールドのデフォルト値を得るために、`Default::default()`または`default = "..."`で与えられた関数を使用します。

### `#[serde(skip_serializing_if = "path")]`

シリアライズしているときにこのフィールドをスキップするかを決定する関数を呼び出します。
与えられた関数は、`fn(&T) -> bool`として呼び出し可能でなくてはならず、ただし`T`に対してジェネリックである可能性があります。
例えば、`skip_serializing_if = "Option::is_none"`は、`None`である`Option`をスキップします。

### `#[serde(serialize_with = "path")]`

その`Serialize`の実装とは異なる関数を使用して、このフィールドをシリアライズします。
与えられた関数は、`fn<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serialize`として呼び出されなくてはならず、ただし`T`に対してジェネリックである可能性があります。
`serialize_with`を使用したフィールドは`Serialize`の実装を要求しません。

### `#[serde(deserialize_with = "path")]`

その`Deserializer`の実装とは異なる関数を使用して、このフィールドをデシリアライズします。
与えられた関数は、`fn<'de, D>(D) -> Result<T, D::Error> where D: Deserializer<'de>`として呼び出されなくてはならず、ただし`T`に対してジェネリックである可能性があります。
`deserialize_with`を使用したフィールドは`Deserialize`の実装を要求しません。

### `#[serde(with = "module")]`

`serialize_with`と`deserialize_with`の組み合わせです。
Serdeは、`serialize_with`関数として`$module::serialize`を、`deserialize_with`関数として`$module::deserialize`を使用します。

### `#[serde(borrow)]`と`#[serde(borrow = "'a + 'b + ...")]`

ゼロコピーデシリアライゼーションを使用して、デシリアライザーからこのフィールドのデータを借用します。
[この例](https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl)を参照してください。

### `#[serde(bound = "T: MyTrait")]`

`Serialize`と`Deserialize`の実装の`where`句です。
これは、現在のフィールドからSerdeが類推したトレイト制約を置き換えます。

シリアライズとデシリアライズで異なる名前を指定できます。

- `#[serde(bound(serialize = "T: MySerTrait"))]`
- `#[serde(bound(deserialize = "T: MyDeTrait"))]`
- `#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]`

### `#[serde(getter = "...")]`

ひとつ以上のプライベートフィールドを持つ[外部の型](https://serde.rs/remote-derive.html)用に`Serialize`を導出するときに使用します。
