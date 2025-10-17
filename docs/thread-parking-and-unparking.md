# スレッドのパークとアンパーク

> ここに記述した内容は、間違っているかもしれません。

この文書では、Rust標準ライブラリにおけるスレッドの`park`関数と`unpark`メソッドについて説明します。内容はRust 1.78以降の仕様に基づいています。

## park関数: Rust標準ライブラリドキュメント

<https://doc.rust-lang.org/std/thread/fn.park.html>

```rust
pub fn park()
```

現在のスレッドの**トークン**が利用可能になるまでブロックします。

`park`を呼び出すことは、スレッドが永遠に停止したままでいることを保証せず、呼び出し側はこの可能性に備えるべきです。
ただし、この関数がパニックしないことは保証されています（実装が何らかな稀なエラーに遭遇した場合、プロセスを中止（abort）するかもしれません）。

### `park`と`unpark`

すべてのスレッドは、[thread::park](https://doc.rust-lang.org/std/thread/fn.park.html)関数と[thread::Thread::unpark](https://doc.rust-lang.org/std/thread/struct.Thread.html#method.unpark)メソッドにより、いくつかの基本的な低水準のブロックのサポートが備わっています。
`park`はカレントスレッドをブロックして、ブロックされたスレッドのハンドルから`unpark`メソッドを呼び出すことで、他のスレッドから再開することができます。

概念的に、それぞれの[Thread](https://doc.rust-lang.org/std/thread/struct.Thread.html)ハンドルは、関連付けられた**トークン**を持っていますが、最初は存在しません。

- `thread::park`関数は、そのスレッドハンドル用のトークンを利用できるようになるまで、現在のスレッドをブロックして、使用可能になった時点で、自動的にそのトークンを消費します。
  `thread::park`関数は、トークンを消費していないにも関わらず戻る（偽の待機解除）可能性があります。
  [thread::park_timeout](https://doc.rust-lang.org/std/thread/fn.park_timeout.html)は同様に行いますが、スレッドをブロックする最大時間を指定できます。
- `Thread`の`unpark`メソッドは、トークンが利用可能でない場合、自動的にトークンを利用可能にします。
  最初、トークンは存在しないため、`unpark`の後の`park`は、2番目の呼び出しがすぐに戻ります。

> 最初に`unpark`を呼び出して、`park`を実行すると、トークンが利用可能であるため、`park`はスレッドをブロックせずにすぐ戻る。

APIは、通常現在のスレッドのハンドルを得て、共有データ構造内にハンドルを配置して、他のスレッドがそれを見つけられるようにして、ループ内で停止するために使用されます。
いくつか望まれた条件に適合したとき、他のスレッドはハンドルに対して[unpark](https://doc.rust-lang.org/std/thread/struct.Thread.html#method.unpark)を呼びます。

この設計の同期は2つあります。

- これは、ミューテックスと新しい同期プリミティブを構築する条件変数を割り当てる必要がないようにします。
  スレッドは基本的なブロックと信号を提供しています。
- これは、多くのプラットフォームで非常に効率的に実装されています。

## 説明

`park`と`unpark`は、通行許可（**トークン**）のように、許可がない場合はてスレッドを停止したり、許可がある場合はスレッドを再開したりする仕組みです。

スレッドは、初期状態でトークンを持っていないため、`unpark`されるとトークンを持ちます。
次に`park`されても、トークンを持っているためスレッドはブロックされませんが（**偽の待機解除**）、トークンは取り消されます（消費されます）。
次に`park`されたとき、トークンを持っていないため、スレッドはブロックされます。
次に、`unpark`されるとトークンを持ち、ブロックが解除され、スレッドが再開されます。

`park`は、トークンが与えられていなくても戻る（再開する）可能性があります。
これは**偽の待機解除（spurious wakeup）**と呼ばれるものであり、プログラム側でループなどを用いて再度条件を確認する必要があります。

`spawn`によってスレッドが生成されても、そのスレッドに最初からトークンが与えられるわけではありません。
`unpark`を呼ぶことで初めてトークンが発行され、対応するスレッドの`park`呼び出しが解除されます。

逆にスレッドが起動後、`park`が発行された場合、トークンがないため、スレッドをブロックします。
この場合、`unpark`が呼ばれるまでスレッドはブロックされたままです。

**初期状態:**

```text
Main Thread           Child Thread
─────────────   ─────────────
トークン: ×             トークン: ×
状態   : 実行中          状態   : 実行中
```

**メインスレッドがparkでブロック**

```text
Main Thread           Child Thread
─────────────   ─────────────
トークン: ×             トークン: ×
状態   : ブロック         状態  : 実行中

説明: トークンがないため、メインスレッドはparkで停止。
```

**子スレッドでトークンを付与**

```text
Main Thread           Child Thread
─────────────   ─────────────
トークン: ✓             トークン: ×
状態   : ブロック        状態   : 実行中

説明: メインスレッドのトークンが利用可能になった。
```

**メインスレッドがparkから復帰**

```text
Main Thread           Child Thread
─────────────   ─────────────
トークン: ×             トークン: ×
状態   : 実行中          状態   : 実行中

説明: parkはトークンを消費して復帰させる。
```

**偽の待機解除（spurious wakeup）の例**

```text
Main Thread           Child Thread
─────────────   ─────────────
トークン: ×             トークン: ×
状態   : ブロック         状態   : 実行中

でも、スレッドは何らかの理由で突然復帰する場合がある

Main Thread            Child Thread
─────────────   ─────────────
トークン: ×             トークン: ×
状態   : 実行中          状態   : 実行中

説明: 偽の待機解除（spurious wakeup）が発生。条件再確認のループが必要。
```

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // メインスレッドのハンドルを取得
    let main_handle = thread::current();

    let child_handle = thread::spawn(move || {
        // 3秒後にメインスレッドを再開
        println!("child thread: sleeping in three seconds");
        thread::sleep(Duration::from_secs(3));
        println!("child thread: get a token to main thread (unpark)");
        main_handle.unpark();
    });

    println!("main thread: blocking for park");
    thread::park(); // トークンがないためブロックされる
    println!("main thread: take a token in child thread, so main thread is resumed");

    child_handle.join().unwrap();
}
```
