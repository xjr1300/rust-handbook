# 18. 列挙型の操作

<https://googleapis.github.io/google-cloud-rust/working_with_enums.html>

このガイドは、Google Cloud Client Libraries for Rustがリリースされた後で導入された列挙型の値を操作する方法を含め、ライブラリ内の列挙型を使用する方法を紹介します。

## 背景

Google Cloudサービスは、限られた離散的な値の集合のみを受け付ける、または提供するフィールドに対して列挙型を使用します。
許可された値の集合はどの時点でも知られている一方で、このリストは時間が経過するにつれて変更される可能性があります。

クライアントライブラリは列挙型を受け取りまたは送信するために準備され、そのライブラリがリリースされた後で導入された値にも機能するようにサポートされています。

## 事前条件

このガイドはGoogle Cloudサービスへの呼び出しを行いません。
Google Cloudのプロジェクトやアカウントを持っていなくても例を実行できます。
このガイドは、[Secret Manager](https://cloud.google.com/secret-manager)用のクライアントライブラリを使用します。
同じ原則は、他のクライアントライブラリのすべての列挙型にも適用できます。

## 依存関係

Rustではいつもの通り、`Cargo.toml`ファイルに依存関係を宣言する必要があります。
次を使用します。

```sh
cargo add google-cloud-secretmanager-v1 serde_json
```

## 知られている値を扱う

知られている値を使用するとき、通常通り列挙型を使用できます。

```rust
    use google_cloud_secretmanager_v1::model::secret_version::State;
    let enabled = State::Enabled;
    println!("State::Enabled = {enabled}");
    assert_eq!(enabled.value(), Some(1));
    assert_eq!(enabled.name(), Some("ENABLED"));

    let state = State::from(1);
    println!("state = {state}");
    assert_eq!(state.value(), Some(1));
    assert_eq!(state.name(), Some("ENABLED"));

    let state = State::from("ENABLED");
    println!("state = {state}");
    assert_eq!(state.value(), Some(1));
    assert_eq!(state.name(), Some("ENABLED"));
    println!("json = {}", serde_json::to_value(&state)?);
```

## 不明な値を扱う

不明な文字列の値を使用するとき、`.value()`メソッドは`None`を返しますが、他はすべて通常通り機能します。

```rust
    use google_cloud_secretmanager_v1::model::secret_version::State;
    use serde_json::json;
    let state = State::from("STATE_NAME_FROM_THE_FUTURE");
    println!("state = {state}");
    assert_eq!(state.value(), None);
    assert_eq!(state.name(), Some("STATE_NAME_FROM_THE_FUTURE"));
    println!("json = {}", serde_json::to_value(&state)?);
    let u = serde_json::from_value::<State>(json!("STATE_NAME_FROM_THE_FUTURE"))?;
    assert_eq!(state, u);
```

同じ原理は不明な整数値にも適用されます。

```rust
    use google_cloud_secretmanager_v1::model::secret_version::State;
    use serde_json::json;
    let state = State::from("STATE_NAME_FROM_THE_FUTURE");
    println!("state = {state}");
    assert_eq!(state.value(), None);
    assert_eq!(state.name(), Some("STATE_NAME_FROM_THE_FUTURE"));
    println!("json = {}", serde_json::to_value(&state)?);
    let u = serde_json::from_value::<State>(json!("STATE_NAME_FROM_THE_FUTURE"))?;
    assert_eq!(state, u);
```

> 注: 上記の2つのコードブロックは内容が同一だが、原文に合わせそのまま残している。

## 更新に対して準備する

上記で言及した通り、クライアントライブラリのRustの列挙型は、将来のリリースで新しいバリアントが追加される可能性があります。
アプリケーションが壊れることを避けるため、これらの列挙型を`#[non_exhaustive]`で印を付けています。

網羅することを求められていない（`non-exhaustive`）列挙型に対して`match`式を使用した場合、`match`式内に[ワイルドカードパターン](https://doc.rust-lang.org/reference/patterns.html#wildcard-pattern)を含める必要があります。
これは、列挙型に新しいバリアントが含められたときに、コンパイルの問題を避けます。

```rust
use google_cloud_secretmanager_v1::model::secret_version::State;
fn match_with_wildcard(state: State) -> anyhow::Result<()> {
    use anyhow::Error;
    match state {
        State::Unspecified => {
            return Err(Error::msg("the documentation says this is never used"));
        }
        State::Enabled => println!("the secret is enabled and can be accessed"),
        State::Disabled => {
            println!("the secret version is not accessible until it is enabled");
        }
        State::Destroyed => {
            println!("the secret is destroyed, the data is no longer accessible");
        }
        State::UnknownValue(u) => {
            println!("unknown State variant ({u:?}) time to update the library");
        }
        _ => return Err(Error::msg("unexpected value, update this code"));
    }
    Ok(())
}
```

それにもかかわらず、少なくとも、コードをテストし、それが更新されなければならないと決定できるように、新しいバリアントが現れると警告またはエラーにしたい場合があります。
そのような場合、[wildcard_enum_match_arm](https://rust-lang.github.io/rust-clippy/master/#wildcard_enum_match_arm) clippy警告を使用することを検討してください。

```rust
    use google_cloud_secretmanager_v1::model::secret_version::State;
    fn match_with_warnings(state: State) -> anyhow::Result<()> {
        use anyhow::Error;
        #[warn(clippy::wildcard_enum_match_arm)]
        match state {
            State::Unspecified => {
                return Err(Error::msg("the documentation says this is never used"));
            }
            State::Enabled => println!("the secret is enabled and can be accessed"),
            State::Disabled => {
                println!("the secret version is not accessible until it is enabled")
            }
            State::Destroyed => {
                println!("the secret is destroyed, the data is no longer accessible")
            }
            State::UnknownValue(u) => {
                println!("unknown State variant ({u:?}) time to update the library")
            }
            _ => {
                // *もし*、CIがclippyの警告をエラーとして取り扱うように含める場合、
                // `unreachable!()`を使用することを検討してください。
                return Err(Error::msg("unexpected value, update this code"));
            }
        };
        Ok(())
    }
```

また、（現在は安定版ではありませんが）[non_exhaustive_omitted_patterns](https://github.com/rust-lang/rust/issues/89554)リントの使用を検討できます。
