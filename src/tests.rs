use super::*;

fn run_test(src_path: &str, exp_path: &str, res_path: &str) {
    let src: Vec<u8> = fs::read(src_path).expect("Error: Test failed to read source file");
    let exp: Vec<u8> = fs::read(exp_path).expect("Error: Test failed to read expected file");

    compile(src, src_path.to_string(), res_path.to_string(), vec![]);
    let run = Command::new(res_path).output().expect("Error: Failed to run executable");
    let stdout_str: String = String::from_utf8(run.stdout.clone()).expect("Error: Failed to convert stdout to string");
    let exp_str: String = String::from_utf8(exp.clone()).expect("Error: Failed to convert expected to string");
    assert_eq!(exp, run.stdout, "{} Error: Unexpected Program output.\nExpected:\n{}\n\nGot:\n{}", src_path, exp_str, stdout_str);

    let _ = Command::new("rm").arg(res_path).output().expect("Error: Failed to delete compiled executable");
}

#[test]
fn test_arithmetic() {
    run_test("./language_tests/arithmetic.lang", "./language_tests/arithmetic.expected", "./test_arithmetic");
}

#[test]
fn test_conditional() {
    run_test("./language_tests/conditional.lang", "./language_tests/conditional.expected", "./test_conditional");
}

#[test]
fn test_variable() {
    run_test("./language_tests/variable.lang", "./language_tests/variable.expected", "./test_variable");
}

#[test]
fn test_function() {
    run_test("./language_tests/function.lang", "./language_tests/function.expected", "./test_function");
}
