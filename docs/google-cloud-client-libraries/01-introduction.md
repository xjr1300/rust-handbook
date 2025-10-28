# イントロダクション

<https://googleapis.github.io/google-cloud-rust/introduction.html>

Google Cloud Client Libraries for Rustは、Google Cloud Serviceとやり取りするためのRustクレートのコレクションです。

このガイドは、クライアントライブラリで特定のアクションを実行する方法を紹介する小さなチュートリアルのシリーズとして整理されています。
ほとんどのGoogle Cloud Serviceは、[AIP](https://google.aip.dev/)と総称される一連のガイドラインに従っています。
これは、クライアントライブラリのサービスとサービスの間の一貫性を向上します。
リソースを削除またはリストする機能は、ほとんど常に同じインターフェースがあります。

## 読者

このガイドは言語とRustエコシステムに親しみのあるRust開発者を対象としています。
Rustとそれがサポートするツールチェインを使用する方法を知っていることを想定しています。

繰り返しになりますが、ほとんどのガイドは、読者が前にGoogleサービスやクライアントライブラリ（Rustまたは他の言語で）を使用したことがないことを想定しています。
しかし、ガイドはプロジェクトとサービスを初期化するサービス特有のチュートリアルを参照することを提案しています。

## サービス特有のドキュメント

これらのガイドは、それぞれのサービスのチュートリアルとして、またGoogle Cloudで動作するRustアプリケーションを設計する方法を示す拡張的なガイドであることを意図していません。
これらはRust用のクライアントライブラリで生産性を高めるスタート地点です。

それぞれのサービスについてより学ぶために<https://cloud.google.com/>のサービスドキュメントを読むことを推奨します。
もし、Google Cloudようにアプリケーションを設計する方法のガイダンスが必要な場合、[Cloud Architecture Center](https://cloud.google.com/architecture)が探しているものになるでしょう。

## バグ報告

クライアントライブラリまたはドキュメントについてのバグを歓迎します。
[GitHub Issues](https://github.com/googleapis/google-cloud-rust/issues)を使用してください。

## ライセンス

クライアントライブラリのソースとそれらのドキュメントは、[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)に基づいてリリースされます。
