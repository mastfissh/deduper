extern crate dupelib;

use std::path::PathBuf;

fn test_case_dir(case: &str) -> PathBuf {
  let mut path = PathBuf::from(file!());
  path.pop();    
  path.push("test_cases");
  path.push(case);
  path
}

#[test]
fn test_base_case() {
    let options = dupelib::Opt{
      debug: false,
      paths: vec![],
      minimum: None,
      output: None,
      timing: false,
    };
    assert_eq!(dupelib::detect_dupes(options), 0);
}

#[test]
fn test_one_file() {
    let path = test_case_dir("one_file");
    let options = dupelib::Opt{
      debug: false,
      paths: vec![path],
      minimum: None,
      output: None,
      timing: false,
    };
    assert_eq!(dupelib::detect_dupes(options), 0);
}

#[test]
fn test_ident_files() {
    let path = test_case_dir("four_ident_files");
    let options = dupelib::Opt{
      debug: false,
      paths: vec![path],
      minimum: None,
      output: None,
      timing: false,
    };
    assert_eq!(dupelib::detect_dupes(options), 2);
}

#[test]
fn test_ident_files_minimum() {
    let path = test_case_dir("four_ident_files");  
    let options = dupelib::Opt{
      debug: false,
      paths: vec![path],
      minimum: Some(2),
      output: None,
      timing: false,
    };
    assert_eq!(dupelib::detect_dupes(options), 1);
}