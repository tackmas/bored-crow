use freedesktop_file_parser::DesktopFile;

#[cfg(feature = "error")]
pub fn is_desktop_input_equal(desktop_files: &Vec<DesktopFile>, input: &Vec<String>) {
    assert_eq!(desktop_files.len(), input.len(), "Desktop files and input are not equal length");

    let mut df_ref: Vec<_> = desktop_files.iter().collect();
    let mut i_ref: Vec<_> = input.iter().collect();

    df_ref.sort_by(|a, b| a.entry.name.default.cmp(&b.entry.name.default));
    i_ref.sort();

    for i in 0..df_ref.len() {
        assert_eq!(*df_ref[i].entry.name.default, *i_ref[i], "Desktop files and input are not equal");
    }
}

#[cfg(feature = "error")]
pub fn is_desktop_lowercase(desktop_files: &Vec<DesktopFile>) {
    for file in desktop_files {
        assert_eq!(file.entry.name.default, file.entry.name.default.to_ascii_lowercase(), "Desktop file names are not lowercase");
    }
}