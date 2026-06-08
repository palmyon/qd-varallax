<div align = "center">
	<picture>
		<img src = "./image/logo.png", width = 30%>
	</picture>
		<h1>QuantumDivision Vector Parallax</h1>
			<a href = "https://www.rust-lang.org/">
				<img src = "https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
			</a>
			<a href = "https://gpuweb.github.io/gpuweb/">
				<img src = "https://img.shields.io/badge/WebGPU-990000?style=for-the-badge&logo=webgpu&logoColor=white">
			</a>
			<a href = "https://wgpu.rs/">
				<img src = "https://img.shields.io/badge/wgpu-green?style=for-the-badge">
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

# 特徴
* Pure Rust
* 5大OS+WASM対応(理論上)
* 完全ネイティブGPU描画
* テキストのMSDF描画
* SDFを使った図形描画
* BVHヒット判定
* スレッドセーフシグナルシステム
* バインドレステクスチャシステムの採用
* バッチング最適化による描画命令の効率化
* ウィジェット、ウィンドウの高い拡張性
* RetainedとImmediateの部分的切り替え機能
* Deriveマクロによるテンプレートの削減

