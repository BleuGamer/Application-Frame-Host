use util::env::get_cwd;
use std::env::current_exe;

#[test]
fn test_env_cwd() {
    let cwd = get_cwd().unwrap();

    let mut rcwd = match current_exe() {
        Ok(exe_path) => exe_path,
        Err(e) => panic!(e),
    };
    rcwd.pop();

    assert_eq!(cwd, rcwd);
}