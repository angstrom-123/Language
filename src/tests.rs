use super::*;

#[test]
fn test_compilation() {
    let tests: [(&'static str, &'static str); 1] = [
        ("./tests/test_arithmetic.lang", "./tests/test_arithmetic.expected"),
    ];
    for test in tests {
        let src_path = test.0;
        let exp_path = test.1;
        let src: String = fs::read_to_string(src_path).expect("Error: Test failed to read source file");
        let exp: String = fs::read_to_string(exp_path).expect("Error: Test failed to read expected file");

        compile(src_path.to_string(), src.clone(), vec![]);
        let run = Command::new("./output").output().expect("Error: Failed to run executable");
        let stdout = String::from_utf8(run.stdout).expect("Error: Failed to convert stdout to string");
        assert_eq!(exp, stdout, "{} Error: Unexpected Program output.\nExpected:\n{}\n\nGot:\n{}", src_path, exp, stdout);
    }
}

