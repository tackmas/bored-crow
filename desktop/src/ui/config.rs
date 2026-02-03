use std::{
    io,
};
use freedesktop_file_parser::DesktopFile;

pub fn _desktop_names(desktop_files: &Vec<DesktopFile>) -> Vec<&String> {
    desktop_files
        .iter()
        .map(|file| &file.entry.name.default)
        .collect()
}


pub fn display_applications(desktop_files: &Vec<DesktopFile>) {
    for file in desktop_files {
        println!("{:?}", file.entry.name.default);
    }
}

pub fn get_input(desktop_files: &Vec<DesktopFile>) -> Vec<String> {
    let mut input = Vec::new();

    println!("What applications would you like to be blocked?\n
Enter each application name per line. Finish by entering 'done' on a new line.");

    #[cfg(feature = "error")]
    crate::error::is_desktop_lowercase(&desktop_files);
    
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Error: could not get input");

        trim_end_in_place(&mut line);
        line.make_ascii_lowercase();

        let mut is_application = false;

        for file in desktop_files {
            if line == file.entry.name.default {
                is_application = true;
            }
        }
        if line == "done" {
            break;
        } 
        else if !is_application {
            println!("Invalid name; application does not exist in your system. Try again");
            continue;
        } else if input.contains(&line) {
            println!("Already entered {line}");
            continue;
        }   
        else {
            input.push(line);
        }
    }

    #[cfg(feature = "error")]
    crate::error::is_desktop_input_equal(&desktop_files, &input);

    input
}

fn trim_end_in_place(s: &mut String) {
    let trimmed = s.trim_end();
    s.truncate(trimmed.len())
}