#[cfg(test)]
mod cli {
    use assert_cmd::Command;
    use std::io::Write;

    fn sd() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Error invoking sd")
    }

    fn assert_file(path: &std::path::Path, content: &str) {
        assert_eq!(content, std::fs::read_to_string(path).unwrap());
    }

    fn create_soft_link<P: AsRef<std::path::Path>>(src: &P, dst: &P) {
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(src, dst).unwrap();

        #[cfg(target_family = "windows")]
        std::os::windows::fs::symlink_file(src, dst).unwrap();
    }

    #[test]
    fn in_place() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc123def").unwrap();
        let path = file.into_temp_path();

        sd().args(["abc\\d+", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path, "def");
    }

    #[test]
    fn in_place_with_empty_result_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"a7c").unwrap();
        let path = file.into_temp_path();

        sd().args(["a\\dc", "", path.to_str().unwrap()])
            .assert()
            .success();
        assert_file(&path, "");
    }

    #[test]
    fn in_place_following_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let file = path.join("file");
        let link = path.join("link");

        create_soft_link(&file, &link);
        std::fs::write(&file, "abc123def").unwrap();

        sd().args(["abc\\d+", "", link.to_str().unwrap()])
            .assert()
            .success();

        assert_file(&file, "def");
        assert!(std::fs::symlink_metadata(link)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn replace_into_stdout() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc123def").unwrap();

        let file_path_str = file.path().to_str().unwrap();

        sd().args(["-p", "abc\\d+", "", file_path_str])
            .assert()
            .success()
            .stdout(format!(
                "----- FILE {} -----\n{}{}def\n",
                file_path_str,
                ansi_term::Color::Green.prefix(),
                ansi_term::Color::Green.suffix()
            ));

        assert_file(file.path(), "abc123def");
    }

    #[test]
    fn stdin() {
        sd().args(["abc\\d+", ""])
            .write_stdin("abc123def")
            .assert()
            .success()
            .stdout("def");
    }
}
