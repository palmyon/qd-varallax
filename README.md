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
			</a>
</div>

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
<a href = "./qd-varallax/src/widgets/text.rs">VxTextWidget</a>を使いテキストを表示しています。
<br><br>

# 特徴
* 純Rust製
* 5大OS+WASM対応(理論上)
* 完全ネイティブGPU描画
* テキストのMSDF描画&効率的な動的パッキング
* SDFを使った図形描画
* BVHヒット判定
* スレッドセーフシグナルシステム
* バインドレステクスチャシステムの採用
* バッチング最適化による描画命令の効率化
* ウィジェット、ウィンドウの高い拡張性
* RetainedとImmediateの部分的切り替え機能
* Deriveマクロによるテンプレートの削減
<br><br>

# クイックスタート
## ※現在開発中のため、APIが予告なく変更される場合があります。
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

* [x] wgpuパイプラインの整備