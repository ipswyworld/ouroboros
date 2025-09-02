use clap::{App, Arg};
use std::fs;
fn main() {
    let matches = App::new("dag-explorer")
      .arg(Arg::with_name("file").short('f').long("file").takes_value(true))
      .get_matches();
    if let Some(f) = matches.value_of("file") {
        let s = fs::read_to_string(f).expect("read file");
        println!("{}", s);
    } else {
        println!("Use -f dag_state.json");
    }
}
