use std::env;

pub(crate) fn get() -> String {
    let process_args: Vec<String> = env::args().collect();
    process_args[0].as_str().to_owned()
}
