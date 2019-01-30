extern crate dupelib;

use std::path::PathBuf;



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
    let path: PathBuf = [ "testdirs", "one_file"].iter().collect();
    dbg!(path.clone().as_os_str());
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
    let mut path = PathBuf::from(file!());
    path.pop();    
    path.push("testdirs");
    path.push("four_ident_files");    
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
    let mut path = PathBuf::from(file!());
    path.pop();    
    path.push("testdirs");
    path.push("four_ident_files");    
    let options = dupelib::Opt{
      debug: false,
      paths: vec![path],
      minimum: Some(2),
      output: None,
      timing: false,
    };
    assert_eq!(dupelib::detect_dupes(options), 1);
}