<div align = "center">
	<picture>
		<img src = "./image/logo.png" width = 30%>
	</picture>
		<h1>QuantumDivision Vector Parallax</h1>
			<a href = "https://www.rust-lang.org/">
				<img src = "https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
			</a>
			<a href = "https://gpuweb.github.io/gpuweb/">
				<img src = "https://img.shields.io/badge/WebGPU-990000?style=for-the-badge&logo=webgpu&logoColor=white">
			</a>
			<a href = "https://wgpu.rs/">
				<img src = "https://img.shields.io/badge/wgpu-green?style=for-the-badge&logo=webgpu">
			</a>
			<br>
			<a href = "https://crates.io/crates/winit">
				<img src = "https://img.shields.io/crates/v/winit?label=winit&style=for-the-badge&logo=rust&color=blue">
			</a>
			<a href = "https://crates.io/crates/swash">
				<img src = "https://img.shields.io/crates/v/swash?label=swash&style=for-the-badge&logo=rust&color=lightblue">
			</a>
			<a href = "https://crates.io/crates/fdsm">
				<img src = "https://img.shields.io/crates/v/fdsm?label=fdsm&style=for-the-badge&logo=rust&color=purple">
			</a><br>
			<a href = "./LICENCE-MIT.txt">
				<img src = "https://img.shields.io/badge/Licence-Mit-yellow.svg?style=for-the-badge">
			</a>
			<a href = "./LICENCE-APACHE.txt">
				<img src = "https://img.shields.io/badge/Licence-Apache_2.0-blue.svg?style=for-the-badge">
			</a>
</div>


<details>
	<summary><b>日本語バージョン(クリックで展開)</b></summary>

# 概要
「QuantumDivision Vector Parallax」<br>
略して「QD-Varallax」は、RustとWebGPUを使用した、超高速マルチプラットフォームGUIフレームワークです。
<br><br>

# スクリーンショット
<picture>
	<img src = "./image/example.png" width="100%">
</picture>
背景のゲーミングカラーは
<a href = "./qd-varallax/src/widgets/vx_widgets.rs">VxRectWidget</a>
を1920個使って描画しています。<br>また、
<a href = "./qd-varallax/src/widgets/button.rs">VxButtonWidget</a>
を配置し、
<a href = "./qd-varallax/src/widgets/text.rs">VxTextWidget</a>
を使いテキストを表示しています。
<br><br>

# 特徴
* **純Rust製**
* **5大OS+WASM対応**(理論上)
* **完全ネイティブGPU描画**
* **テキストのMSDF描画&効率的な動的パッキング**
* **SDFを使った図形描画**
* **BVHヒット判定**
* **スレッドセーフシグナルシステム**
* **バインドレステクスチャシステムの採用**
* **バッチング最適化による描画命令の効率化**
* **ウィジェット、ウィンドウの高い拡張性**
* **RetainedとImmediateの部分的切り替え機能**
* **Derive、macro_rules!マクロによるテンプレートの削減**
<br><br>

# アーキテクチャ
QD-Varallaxでは、大まかに以下の図(英語版と共通)のようなアーキテクチャを構築しています。
<br><br>
<picture>
	<img src = "./image/qd-varallax-architecture.png">
</picture>
## 主要コンポーネント
* <b><a href = "./qd-varallax/src/core/application.rs">VxApplication</a></b> - winitのイベントループとウィンドウイベントを管理し、それぞれのウィンドウに振り分けます。
* <b><a href = "./qd-varallax/src/core/resource.rs">VxAppResources</a></b> - wgpuのリソースを管理し、アプリ共通のデータを保有&管理しています。
* <b><a href = "./qd-varallax/src/core/renderer.rs">VxRenderer</a></b> - 
<a href = "./qd-varallax/src/abstractions/abstract_windows.rs">VxWindowStats</a>から
受け取った頂点をソートしてバッファに書き込み、バッチング描画を実行するコンポーネントです。
* <b><a href = "./qd-varallax/src/abstractions/abstract_windows.rs">VxWindowStats</a></b> - ウィンドウ単位のリソースを管理し、ウィンドウ本体のデータとして動作します。
* <b><a href = "./qd-varallax/src/painter/painter.rs">VxPainter</a></b> - 描画ループごとに作成され、paintループでウィジェットの頂点や描画データを作成します。
* <b><a href = "./qd-varallax/src/core/scene.rs">VxScene</a></b> - ウィジェット本体を管理し、ウィジェットへ各イベントを配信するコンポーネントです。
* <b><a href = "./qd-varallax/src/abstractions/abstract_widgets.rs">VxWidget</a></b> - ウィジェット本体となるトレイトです。このトレイトを継承し、データを持たせることで、ウィジェットとして動作します。

## 主要機能
### アプリケーション
* `winit`のイベントを管理し、各ウィンドウに適切なイベントを変換して振り分けます。
* アプリケーション全体のリソース(`VxAppResources`)を保有し、必要となるイベントで参照を渡します。
* 全てのウィンドウを管理し、描画チェックのループや、アプリケーションの終了などを自動で行います。
### ウィジェット

# クイックスタート
<a href = "./qd-varallax/src/widgets/default_window.rs">デフォルトの空のウィンドウ</a>
を使用してウィンドウを出す最小構成例です。

```rust
// Windowsでリリースビルド時に、ターミナルを非表示にする設定(オヌヌメ)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qd_varallax::{
	core::application::VxApplication,
	widgets::default_window::VxDefaultWindow,
};

fn main() {
	// アプリケーションを作成
	let mut app = VxApplication::new();

	// ウィンドウを作成
	let window = VxDefaultWindow::new(Default::default());

	// ウィンドウを初期ウィンドウとして登録
	app.add_window(window);

	// アプリケーションループを開始
	app.exec();
}
```
<br>

# ロードマップ
現状実装できているもの、できていないものについてです。

### できているもの
* [x] wgpuパイプラインの整備
* [x] テキストのMSDF描画&アトラスパッキングアルゴリズム
* [x] SDFを使った図形描画
* [x] BVHヒット判定
* [x] スレッドセーフシグナルシステムロードマップ

### できていないもの
* [ ] 5大OS+WASM対応
* [ ] RetainedとImmediateの部分的切り替え機能
* [ ] IME入力への対応
* [ ] レイアウト機能
* [ ] アクセシリビティへの対応
* [ ] 細かなウィジェットの実装
* [ ] そのほか諸々！！！
<br><br>

# ライセンス
QD-Varallaxは、以下のデュアルライセンスのもとで提供されています。

* **Apache License, Version 2.0** ([LICENSE-APACHE](/LICENSE-APACHE.txt)、または http://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](./LICENSE-MIT)、または http://opensource.org/licenses/MIT)

お好きな方を選んでお好きなようにお使いください。
</details>