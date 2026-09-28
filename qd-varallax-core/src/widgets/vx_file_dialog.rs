
/// QD-Varallax VxWidgets> VxFileDialog> method> get_open_file_path() \
/// ファイルダイアログを開き、選択されたファイルのパスを取得する。 \
/// * # Arguments
/// * `title`: ダイアログのタイトル \
/// * `filters`: ファイルフィルター。`(ファイルの分類, [拡張子])` \
/// * `start_dir`: 開いたときのディレクトリパス \
/// 
/// # Returns
/// ファイルが選択された場合`Some(PathBuf)`, キャンセルした場合`None`を返す。
/// 
/// # Examples
/// ```no_run
/// let file = vx_file_dialog::get_open_file_path(
/// 	"Select File",
/// 	vec![
/// 		("Images", &["png", "jpg", "webp"]),
/// 		("Sounds", &["flac", "wav", "mp3"]),
/// 		("All Files", &["*"]),
/// 	],
/// 	"C:/"
/// );
/// if let Some(path) = file {
/// 	println!("Selected: {:?}", path);
/// } else {
/// 	println!("Selection was canceled.");
/// }
/// ```
pub fn get_open_file_path(
	title: impl Into<String>,
	filters: Vec<(impl Into<String>, &[impl ToString])>,
	start_dir: impl Into<std::path::PathBuf>,
) -> Option<std::path::PathBuf> {
	let mut dialog = rfd::FileDialog::new()
			.set_title(title)
			.set_directory(start_dir.into());
	for (name, exts) in filters {
		dialog = dialog.add_filter(name, exts);
	}
	dialog.pick_file()
}

/// QD-Varallax VxWidgets> VxFileDialog> method> get_open_file_list() \
/// ファイルダイアログを開き、選択されたファイルのパスを複数取得する。 \
/// * # Arguments
/// * `title`: ダイアログのタイトル \
/// * `filters`: ファイルフィルター。`(ファイルの分類, [拡張子])` \
/// * `start_dir`: 開いたときのディレクトリパス \
/// 
/// # Returns
/// ファイルが1個以上選択された場合`Some(Vec<PathBuf>)`, キャンセルした場合`None`を返す。
/// 
/// # Examples
/// ```no_run
/// let files = vx_file_dialog::get_open_file_list(
/// 	"Select File",
/// 	vec![
/// 		("Images", &["png", "jpg", "webp"]),
/// 		("Sounds", &["flac", "wav", "mp3"]),
/// 		("All Files", &["*"]),
/// 	],
/// 	"C:/"
/// );
/// if let Some(file) = files {
/// 	for (i, p) in file.iter().enumerate() {
/// 		println!("Selected path> number: {}, path: {:?}", i, p);
/// 	}
/// } else {
/// 	println!("Selection was canceled.");
/// }
/// ```
pub fn get_open_file_list(
	title: impl Into<String>,
	filters: Vec<(impl Into<String>, &[impl ToString])>,
	start_dir: impl Into<std::path::PathBuf>,
) -> Option<Vec<std::path::PathBuf>> {
	let mut dialog = rfd::FileDialog::new()
			.set_title(title)
			.set_directory(start_dir.into());
	for (name, exts) in filters {
		dialog = dialog.add_filter(name, exts);
	}
	dialog.pick_files()
}

/// QD-Varallax VxWidgets> VxFileDialog> method> get_existing_directory() \
/// ファイルダイアログを開き、選択されたフォルダのパスを取得する。 \
/// * # Arguments
/// * `title`: ダイアログのタイトル \
/// * `start_dir`: 開いたときのディレクトリパス \
/// 
/// # Returns
/// フォルダが選択された場合`Some(PathBuf)`, キャンセルした場合`None`を返す。
/// 
/// # Examples
/// ```no_run
/// let folder = vx_file_dialog::get_existing_directory("Select Folder", "C:/");
/// 
/// if let Some(f) = folder {
/// 	println!("Selected folder path: {:?}", f);
/// } else {
/// 	println!("Selection was canceled.");
/// }
/// ```
pub fn get_existing_directory(
	title: impl Into<String>,
	start_dir: impl Into<std::path::PathBuf>,
) -> Option<std::path::PathBuf> {
	rfd::FileDialog::new()
			.set_title(title)
			.set_directory(start_dir.into())
			.pick_folder()
}

/// QD-Varallax VxWidgets> VxFileDialog> method> get_existing_directory_list() \
/// ファイルダイアログを開き、選択されたフォルダのパスを複数取得する。 \
/// * # Arguments
/// * `title`: ダイアログのタイトル \
/// * `start_dir`: 開いたときのディレクトリパス \
/// 
/// # Returns
/// フォルダが選択された場合`Some(Vec<PathBuf>)`, キャンセルした場合`None`を返す。
/// 
/// # Examples
/// ```no_run
/// let folders = vx_file_dialog::get_existing_directory_list("Select Folders", "C:/");
/// 
/// if let Some(folder) = folders {
/// 	for (i, f) in folder.iter().enumerate() {
/// 		println!("Selected folder path> number: {}, path: {:?}", i, f);
/// 	}
/// } else {
/// 	println!("Selection was canceled.");
/// }
/// ```
pub fn get_existing_directory_list(
	title: impl Into<String>,
	start_dir: impl Into<std::path::PathBuf>
) -> Option<Vec<std::path::PathBuf>> {
	rfd::FileDialog::new()
			.set_title(title)
			.set_directory(start_dir.into())
			.pick_folders()
}

/// QD-Varallax VxWidgets> VxFileDialog> method> get_save_file_path() \
/// ファイルダイアログを開き、ファイルを作成し保存する。 \
/// * # Arguments
/// * `title`: ダイアログのタイトル \
/// * `filters`: ファイルフィルター。`(ファイルの分類, [拡張子])` \
/// * `default_name`: デフォルトのファイル名 \
/// * `start_dir`: 開いたときのディレクトリパス \
/// 
/// # Returns
/// 上書きが選択、もしくは新たに作成された場合`Some(PathBuf)`, キャンセルした場合`None`を返す。
/// 
/// # Examples
/// ```no_run
/// let path = vx_file_dialog::get_save_file_path(
/// 	"Save File",
/// 	vec![
/// 		("Images", &["png", "jpg", "webp"]),
/// 		("Sounds", &["flac", "wav", "mp3"]),
/// 		("All Files", &["*"]),
/// 	],
/// 	"file.txt",
/// 	"C:/"
/// );
/// if let Some(p) = path {
/// 	println!("Saved path: {:?}", p);
/// } else {
/// 	println!("Save was canceled.");
/// }
/// ```
pub fn get_save_file_path(
	title: impl Into<String>,
	filters: Vec<(impl Into<String>, &[impl ToString])>,
	default_name: impl Into<String>,
	start_dir: impl Into<std::path::PathBuf>,
) -> Option<std::path::PathBuf> {
	let mut dialog = rfd::FileDialog::new()
			.set_title(title)
			.set_file_name(default_name)
			.set_directory(start_dir.into());
	for (name, exts) in filters {
		dialog = dialog.add_filter(name, exts);
	}
	dialog.save_file()
}