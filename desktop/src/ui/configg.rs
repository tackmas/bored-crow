use std::io;


use crate::platform::App;

pub fn display_applications(apps: &Vec<App>) {
    for app in apps {
        println!("{app:?}");
    }
}

pub fn prompt_block_selection(apps: &Vec<App>) -> Vec<App> {
    let app_names: Vec<&String> = apps
        .iter()
        .map(|app| app.name())
        .collect();

    let mut input = Vec::with_capacity(app_names.len());

    println!("Which apps would you like to block? Please enter the apps you wish to block exactly as written, line by line.
Once you are finished, enter \"Finish\" on a new line\n");

    for name in &app_names {
        println!{"Apps:\n{}", *name}
    }

    println!("\nEnter:");

    loop {
        let mut line = String::new();

        io::stdin().read_line(&mut line).unwrap();

        trim_in_place(&mut line);

        if line.to_ascii_lowercase() == "finish" {
            break;
        }

        let app_exists = app_names.contains(&&line);

        if !app_exists {
            println!{"Invalid input; the app \"{line}\" does not exist on this platform. 
Enter one of the apps listed above exactly as written, or \"Finish\" if you are done"};
            continue;
        } 

        println!("\nYou have entered \"{line}\"");

        let app = App::from(line);

        input.push(app);

        println!("\nCurrent list of apps you have entered:");

        for app in &input {
            println!("{}", app.name());
        }

        println!("\nEnter \"Finish\" if you are done, or enter more apps: ")
    }

    println!("\nBlocking the apps you have listed:");

    for app in &input {
        println!("{}", app.name());
    }

    input
}

fn trim_in_place(line: &mut String) {
    let trimmed = line.trim_end();

    line.truncate(trimmed.len());
}