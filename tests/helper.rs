#[allow(unused_macros)]
macro_rules! base_dir {
    () => {
        "target/test.out"
    };
}

#[allow(unused_macros)]
macro_rules! open_test_file {
    ($fnm:expr) => {{
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/", $fnm);
        let f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        f
    }};
}
