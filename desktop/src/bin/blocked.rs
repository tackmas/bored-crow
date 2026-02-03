use std::env;

const POSTFIX_LEN: usize = "name_arg".len();

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let arg = find_name_in_args(&mut args);
    let name = trim(arg);

    println!("[{name}] is blocked by Bored Crow");
}

fn find_name_in_args(args: &Vec<String>) -> &String {
    for arg in args {
        let len = arg.len();

        if arg[len-POSTFIX_LEN..].contains("name_arg") {
            return arg
        }
    };

    panic!("Provided no valid, required args");
}   

fn trim(arg: &String) -> &str {
    let len = arg.len();
    &arg[..len-POSTFIX_LEN]
}