use lang_c::driver::parse;

fn analyze_file() {
    let config = lang_c::driver::Config::with_gcc();
    let ast = parse(&config, "import_src/test.c").expect("Failed to parse C Code");
}

fn analyze(tree: ) {}
