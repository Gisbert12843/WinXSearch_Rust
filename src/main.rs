use std::env;

use functions::start_win_x_search;
#[macro_use]
extern crate lazy_static;

mod filetype_searches;
mod functions;
mod helper_functions;

fn main()
{
    let main_args: Vec<String> = env::args().collect();
    dbg!(&main_args);

    let mut search_folders = false; //option for user to search for folders too
    let mut search_content = false; //option for user to search through file content too,

    let mut vec_searchvalue: Vec<String> = Vec::new();

    for i in 2..(main_args.len())
    {
        if main_args[i] == "-f" || main_args[i] == "-F"
        {
            search_folders = true;
            println!("Searching for Foldernames - Activated");
        }
        else if main_args[i] == "-c" || main_args[i] == "-C"
        {
            search_content = true;
            println!("Searching in Filecontent - Activated");
        }
    }
    println!("Please provide your search Strings. Press Enter after every String. Empty Input == Continue with Search.");

    let stdin = std::io::stdin(); // Obtain the stdin handle once

    loop
    {
        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read line");
        let line = input.trim().to_string();
        if line.is_empty()
        {
            break;
        }
        vec_searchvalue.push(line);
    }

    let search_path = main_args[1].to_string();

    // let search_path = "C:\\Users\\Kai\\Sciebo\\Projects".to_string();
    // let search_path = "C:\\Users\\Kai\\Sciebo\\Projects\\WinXSearch".to_string();

    // search_folders = true;
    // search_content = true;

    start_win_x_search(search_path, search_folders, search_content, vec_searchvalue);
    helper_functions::wait_for_user_continue();
}
