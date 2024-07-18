use std::fs::OpenOptions;
// use std::fs::{DirEntry, Metadata};
use std::io::{BufRead, BufReader, Read};
use std::process::Command;
use std::thread::JoinHandle;
// use std::str::SplitTerminator;
use encoding_rs::UTF_8;
use std::sync::{Arc, Mutex};
// use std::{clone, fs, vec};
use crossterm::{cursor, ExecutableCommand};
use pdf_extract::extract_text;
use std::io::{stdout, Write};
use std::{path::Path, thread};
use walkdir::WalkDir;

use crate::filetype_searches;
use crate::helper_functions;
use helper_functions::sort_path_vectors;

// #[macro_use]
extern crate lazy_static;

pub fn display_results(
    seconds: u128,
    total_file_count: i64,
    total_folder_count: i64,
    vec_file_path: &mut Vec<walkdir::DirEntry>,
    vec_skipped_files: &mut Vec<walkdir::DirEntry>,
    vec_folder_path: &mut Vec<walkdir::DirEntry>,
    vec_content_path: &mut Vec<walkdir::DirEntry>,
)
{
    let vec_to_be_opened: Vec<u64> = Vec::new();

    let mut i = 0;

    println!(
      "\nProcessed an astounding {} files (of {}) inside of {} folders in just {} seconds.\nSkipped Files : {}\n\n",
      total_file_count as usize - vec_skipped_files.len(),
      total_file_count,
      total_folder_count,
      seconds / 1000,
      vec_skipped_files.len()
  );

    sort_path_vectors(&mut [
        &mut *vec_file_path,
        &mut *vec_skipped_files,
        &mut *vec_folder_path,
        &mut *vec_content_path,
    ]);

    if !vec_folder_path.is_empty()
    {
        println!("Found Folders\n****************************************************************");
        for (_, entry) in vec_folder_path.iter().enumerate()
        {
            i += 1;
            let spaces =
                " ".repeat(vec_folder_path.len().to_string().len() - (i + 1).to_string().len());
            println!("{}: {}{}", i, spaces, entry.path().to_str().unwrap());
        }
        println!("****************************************************************\n");
    }

    if !vec_file_path.is_empty()
    {
        println!("Found Files\n****************************************************************");
        for (_, entry) in vec_file_path.iter().enumerate()
        {
            i += 1;
            let spaces = " ".repeat(vec_file_path.len() - (i + 1).to_string().len());
            println!("{}: {}{}", i, spaces, entry.path().to_str().unwrap());
        }
        println!("****************************************************************\n");
    }

    if !vec_content_path.is_empty()
    {
        println!("Found Content\n****************************************************************");
        for (_, entry) in vec_content_path.iter().enumerate()
        {
            i += 1;
            let spaces = " ".repeat(vec_content_path.len() - (i + 1).to_string().len());
            println!("{}: {}{}", i, spaces, entry.path().to_str().unwrap());
        }
        println!("****************************************************************\n");
    }

    println!("Skipped Files:");
    if vec_skipped_files.len() > 30
    {
        helper_functions::clear_log_file();

        #[cfg(debug_assertions)]
        let log_path = helper_functions::synchronized_println_with_string(
            "Too many files to display. Please check the log file for more information."
                .to_string(),
        );

        match std::fs::File::create(
            helper_functions::get_log_file().expect("Failed to create log file"),
        )
        {
            Ok(log_file) =>
            {
                let mut file = log_file;

                let mut log_string = String::new();
                for i in vec_skipped_files.iter()
                {
                    log_string.push_str(i.path().to_str().unwrap());
                    log_string.push_str("\n");
                }

                file.write_all(log_string.as_bytes()).unwrap();
                helper_functions::synchronized_println_with_string(
                    "Log file created successfully at: ".to_string()
                        + helper_functions::get_log_file()
                            .expect("Failed to create log file")
                            .to_str()
                            .unwrap(),
                );
            }
            Err(_) =>
            {
                helper_functions::synchronized_eprintln_with_string(
                    "Failed to create log file.".to_string()
                        + helper_functions::get_log_file()
                            .expect("Failed to create log file")
                            .to_str()
                            .unwrap(),
                );
            }
        }
    }
    else
    {
        for entry in vec_skipped_files.iter()
        {
            println!("{}", entry.path().to_str().unwrap());
        }
    }

    println!("\n\n");

    let mut to_be_opened: Vec<String> = Vec::new();

    //Collect user input for which files to open, if files were found
    if !(vec_content_path.is_empty() && vec_file_path.is_empty() && vec_folder_path.is_empty())
    {
        let stdin = std::io::stdin();

        helper_functions::synchronized_println(format_args!("What should be opened? (Comma separated list of numbers, empty input == continue)\nInput-Example: \"1,2,3,40,53\""));
        loop
        {
            let mut input = String::new();
            stdin.read_line(&mut input).expect("Failed to read line");
            let line = input.trim().to_string();
            if line.is_empty()
            {
                break;
            }

            let mut split = line.split_terminator(",");
            while let Some(part) = split.next()
            {
                match part.trim().parse::<u64>()
                {
                    Ok(num) =>
                    {
                        if num <= i
                        {
                            to_be_opened.push(part.to_string())
                        }
                    }

                    Err(_) => helper_functions::synchronized_println(format_args!(
                        "Invalid Input: {}",
                        part
                    )),
                }
            }
        }
    }
    else
    //else we just wait for the user to press enter and abort the programm
    {
        println!("No files match the searching criteria :(");
        println!("Press \"Enter\" to exit.");
        let stdin = std::io::stdin();
        loop
        {
            let mut input = String::new();
            stdin.read_line(&mut input).expect("Failed to read line");
            std::process::abort();
        }
    }

    println!("Opening Files/Folders/Content...");
    println!("vec_to_be_opened size: {}", to_be_opened.len());

    for it in 0..to_be_opened.len() as u64
    {
        println!("{} >= {}", vec_folder_path.len(), it);
        println!(
            "{} >= {}",
            vec_file_path.len() as u64 + vec_folder_path.len() as u64,
            it
        );
        println!(
            "{} >= {}",
            vec_content_path.len() as u64
                + vec_file_path.len() as u64
                + vec_folder_path.len() as u64,
            it
        );

        if vec_folder_path.len() as u64 >= it
        {
            println!(
                "Opening Folder: {}",
                vec_folder_path[(it - 1) as usize].path().display()
            );
            let x = &vec_folder_path[(it - 1) as usize];
            // println!("{:?}", x);
            let askdjasd = format!("{}\\\\", x.path().display());
            let y = askdjasd.as_str();
            Command::new("cmd").arg("/C").arg(y).status().unwrap();
            // helper_functions::browse_to_file(
            //     &vec_folder_path[(it - 1) as usize]
            //         .path()
            //         .display()
            //         .to_string()
            //         .as_str(),
            // );
        }
        else if (vec_file_path.len() as u64 + vec_folder_path.len() as u64) >= it
        {
            println!(
                "Opening File: {}",
                vec_file_path[(it - 1) as usize].path().display()
            );
            helper_functions::browse_to_file(
                &vec_file_path[(it - 1) as usize]
                    .path()
                    .display()
                    .to_string()
                    .as_str(),
            );
        }
        else if (vec_content_path.len() as u64
            + vec_file_path.len() as u64
            + vec_folder_path.len() as u64)
            >= it
        {
            println!(
                "Opening Content: {}",
                vec_content_path[(it - 1) as usize].path().display()
            );
            helper_functions::browse_to_file(
                &vec_content_path[(it - 1) as usize]
                    .path()
                    .display()
                    .to_string()
                    .as_str(),
            );
        }
    }
}

pub fn big_loop(
    p_i: i32,                                              // The index of the current thread
    total_threads: i32,                                    // The total number of threads to be used
    path_to_folder: String,                                // The path to the folder being searched
    do_search_folders: bool,                               // Whether to search folder names
    do_search_content: bool,                               // Whether to search file content
    processed_files: Arc<Mutex<i64>>, // Shared counter for processed files (includes skipped files)
    processed_folders: Arc<Mutex<i64>>, // Shared counter for processed folders
    vec_skipped_files: Arc<Mutex<Vec<walkdir::DirEntry>>>, // Shared vector of skipped files
    vec_search_value: Arc<Mutex<Vec<String>>>, // Shared vector of search values
    vec_file_path: Arc<Mutex<Vec<walkdir::DirEntry>>>, // Shared vector of file paths that match search criteria
    vec_folder_path: Arc<Mutex<Vec<walkdir::DirEntry>>>, // Shared vector of folder paths that match search criteria
    vec_content_path: Arc<Mutex<Vec<walkdir::DirEntry>>>, // Shared vector of content paths that match search criteria
)
{
    let mut local_skipped_files = Vec::new(); // Local vector to hold skipped files for this thread

    let vec_search_value_copy = vec_search_value.lock().unwrap().clone();

    // Create an iterator to walk through the directory
    let mut iterator = WalkDir::new(path_to_folder)
        .into_iter() // Create an iterator over directory entries
        .filter_map(|e| match e
        {
            Ok(entry) => Some(entry),
            Err(err) =>
            {
                helper_functions::synchronized_eprintln(format_args!(
                    "Error iterating entry: {} | Reason: {}\n",
                    err,
                    err.io_error().unwrap().to_string()
                ));
                None
            }
        })
        .enumerate() // Enumerate the entries to get an index for each one
        .filter(|(index, _)| index % total_threads as usize == p_i as usize - 1) // Filter entries so that each thread processes a subset of the entries
        .map(|(_, entry)| entry) // Map back to the directory entries
        .peekable(); // Convert the iterator to a peekable iterator

    // Process each entry in the iterator
    while let Some(entry) = iterator.next()
    {
        // #[cfg(debug_assertions)]
        // helper_functions::synchronized_println(format_args!(
        //     "Thread: {} Trying File: {}",
        //     p_i,
        //     entry.path().to_str().unwrap()
        // ));

        // If the entry is a directory
        if entry.path().is_dir()
        {
            // #[cfg(debug_assertions)]
            // helper_functions::synchronized_println(format_args!(
            //     "Thread: {} Processing Folder: {}",
            //     p_i,
            //     entry.path().to_str().unwrap()
            // ));
            *processed_folders.lock().unwrap() += 1; // Increment the processed folder count

            // If we are not searching folders, continue to the next entry
            if !do_search_folders
            {
                continue;
            }

            // Check if the folder name contains any of the search values
            for search_value in vec_search_value.lock().unwrap().iter()
            {
                if entry.path().to_str().unwrap().contains(search_value)
                {
                    vec_folder_path.lock().unwrap().push(entry); // Add the entry to the folder path vector
                    break;
                }
            }
            continue;
        }
        else if entry.path().is_file()
        {
            // If the entry is a file
            // #[cfg(debug_assertions)]
            // helper_functions::synchronized_println(format_args!(
            //     "Thread: {} Processing File: {}",
            //     p_i,
            //     entry.path().to_str().unwrap()
            // ));
            *processed_files.lock().unwrap() += 1; // Increment the processed file count
                                                   // println!("Processed Files now: {}", *processed_files.lock().unwrap());

            let mut found = false; // Variable to track if a match is found

            // Check if the file name contains any of the search values
            for search_value in vec_search_value.lock().unwrap().iter()
            {
                if entry.file_name().to_str().unwrap().contains(search_value)
                {
                    vec_file_path.lock().unwrap().push(entry.clone()); // Add the entry to the file path vector
                    found = true; // Set found to true
                    break;
                }
            }

            // If a match was found or we are not searching file content, continue to the next entry
            if found || !do_search_content
            {
                continue;
            }

            let extension = entry.path().extension().and_then(std::ffi::OsStr::to_str);

            match extension
            {
                Some("pdf") =>
                {
                    let result =
                        filetype_searches::search_pdf_file(&entry, vec_search_value.clone());
                    match result
                    {
                        Ok(result) =>
                        {
                            match result
                            {
                                true =>
                                {
                                    vec_content_path.lock().unwrap().push(entry); // Add the entry to the content path vector
                                    continue;
                                }
                                false => continue,
                            }
                        }

                        Err(e) =>
                        {
                            #[cfg(debug_assertions)]
                            helper_functions::synchronized_eprintln(format_args!(
                                "Error extracting PDF-File: {} | Reason: {}\n",
                                entry.path().to_str().unwrap(),
                                e.to_string()
                            ));
                            local_skipped_files.push(entry); // Add the entry to the local skipped files vector
                            continue;
                        }
                    }
                }
                // Some("doc") =>
                // {}
                // Some("docx") =>
                // {}
                _ =>
                {
                    let file = std::fs::File::open(entry.path());
                    match file
                    {
                        Ok(_) => (),
                        Err(e) =>
                        {
                            #[cfg(debug_assertions)]
                            helper_functions::synchronized_eprintln(format_args!(
                                "Error opening File: {} | Reason: {}\n",
                                entry.path().to_str().unwrap(),
                                e.to_string()
                            ));

                            local_skipped_files.push(entry); // Add the entry to the local skipped files vector
                            continue;
                        }
                    }

                    let file = file.unwrap();
                    let reader = BufReader::new(&file); // Create a buffered reader for the file
                                                        // Read each line of the file
                    for line in reader.lines()
                    {
                        match line
                        {
                            Ok(line) =>
                            {
                                // Check if the line contains any of the search values

                                for search_value in vec_search_value_copy.iter()
                                {
                                    if line.contains(search_value)
                                    {
                                        vec_content_path.lock().unwrap().push(entry.clone()); // Add the entry to the content path vector
                                        found = true; // Set found to true
                                        break;
                                    }
                                }
                                if found
                                {
                                    break;
                                }
                            }
                            Err(_) =>
                            {
                                continue;
                            }
                        }
                    }
                }
            }
            // Try to open the file
        }
    }

    // Merge the local skipped files vector with the shared skipped files vector
    vec_skipped_files
        .lock()
        .unwrap()
        .extend(local_skipped_files);
}

pub fn print_progress(_: i32, processed_files: Arc<Mutex<i64>>, total_files: Arc<Mutex<i64>>)
{
    helper_functions::clear_screen();
    let mut percentage: i64 = 0;
    let mut last_print = String::new();

    while percentage < 100
    {
        let mut max_print: i8 = 20;
        percentage =
            100 * *processed_files.lock().unwrap() as i64 / *total_files.lock().unwrap() as i64;

        let to_print = (percentage / 5) as i64;

        let mut print = String::new();

        // print += "to_print: ";
        // print += to_print.to_string().as_str();
        // print += "\n";

        // print += "percentage: ";
        // print += percentage.to_string().as_str();
        // print += "\n";

        // print += "total_files: ";
        // print += total_files.lock().unwrap().to_string().as_str();
        // print += "\n";

        // print += "processed_files:";
        // print += processed_files.lock().unwrap().to_string().as_str();
        // print += "\n";

        print += "[";
        for _ in 0..to_print
        {
            print += "|";
            max_print -= 1;
        }

        for _ in 0..max_print
        {
            print += " ";
        }
        print += "]";
        print += &percentage.to_string();
        print += "%\n";

        stdout().execute(cursor::MoveTo(0, 0)).unwrap();

        println!("{}", print);

        if last_print.len() > print.len()
        {
            for _x in 0..(last_print.len() - print.len())
            {
                print!(" ");
            }
        }
        last_print = print;
        thread::sleep(std::time::Duration::from_millis(500));
    }
    helper_functions::clear_screen();
}

pub fn start_win_x_search(
    search_path_str: String,
    search_folders: bool,
    search_content: bool,
    vec_searchvalue: Vec<String>,
)
{
    // is the Path valid?
    if !Path::new(&search_path_str).exists()
    {
        #[cfg(debug_assertions)]
        helper_functions::synchronized_println_with_string(
            "Path does not exist. Quitting.".to_string(),
        );
        helper_functions::wait_for_user_continue();
        std::process::exit(0);
    }

    let start_time = std::time::Instant::now();

    println!("Starting Search in: {}", search_path_str);
    let search_path = Path::new(&search_path_str);

    let total_file_count: Arc<Mutex<i64>> = Arc::new(Mutex::new(0_i64));
    let total_folder_count: Arc<Mutex<i64>> = Arc::new(Mutex::new(0_i64));
    let processed_files: Arc<Mutex<i64>> = Arc::new(Mutex::new(0_i64));
    let processed_folders: Arc<Mutex<i64>> = Arc::new(Mutex::new(0_i64));
    let vec_skipped_files: Arc<Mutex<Vec<walkdir::DirEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let vec_search_value: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec_searchvalue));
    let vec_file_path: Arc<Mutex<Vec<walkdir::DirEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let vec_folder_path: Arc<Mutex<Vec<walkdir::DirEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let vec_content_path: Arc<Mutex<Vec<walkdir::DirEntry>>> = Arc::new(Mutex::new(Vec::new()));

    println!("Scanning Files...");

    WalkDir::new(search_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            if entry.path().is_file()
            {
                *(total_file_count.lock().unwrap()) += 1;
            }
            else if entry.path().is_dir()
            {
                *(total_folder_count.lock().unwrap()) += 1;
            }
        });
    *(total_folder_count.lock().unwrap()) -= 1;

    println!("Total Files: {}", total_file_count.lock().unwrap());
    println!("Total Folders: {}", total_folder_count.lock().unwrap());

    let max_thread_count = num_cpus::get() as i32;
    println!("Found {} CPUs.", max_thread_count);

    let mut thread_handles_vec: Vec<std::thread::JoinHandle<()>> = Vec::new();
    let print_thread: JoinHandle<()>;
    {
        let processed_files = processed_files.clone();
        let total_file_count = total_file_count.clone();
        print_thread = thread::spawn(move || {
            print_progress(max_thread_count, processed_files, total_file_count);
        });
    }

    for current_thread_count in 2..=max_thread_count
    {
        let search_path_str = search_path_str.clone(); // Clone the string for each thread
        let processed_files = processed_files.clone();
        let processed_folders = processed_folders.clone();
        let vec_skipped_files = vec_skipped_files.clone();
        let vec_search_value = vec_search_value.clone();
        let vec_file_path = vec_file_path.clone();
        let vec_folder_path = vec_folder_path.clone();
        let vec_content_path = vec_content_path.clone();

        #[cfg(debug_assertions)]
        println!("Starting Thread: {}", current_thread_count);
        thread_handles_vec.push(thread::spawn(move || {
            big_loop(
                current_thread_count,
                max_thread_count,
                search_path_str,
                search_folders,
                search_content,
                processed_files,
                processed_folders,
                vec_skipped_files,
                vec_search_value,
                vec_file_path,
                vec_folder_path,
                vec_content_path,
            )
        }));
    }

    for handle in thread_handles_vec
    {
        handle.join().unwrap();
    }
    *processed_files.lock().unwrap() = *total_file_count.lock().unwrap();

    print_thread.join().unwrap();

    display_results(
        start_time.elapsed().as_millis(),
        *total_file_count.lock().unwrap(),
        *total_folder_count.lock().unwrap(),
        &mut vec_file_path.lock().unwrap(),
        &mut vec_skipped_files.lock().unwrap(),
        &mut vec_folder_path.lock().unwrap(),
        &mut vec_content_path.lock().unwrap(),
    );

    println!("Search took: {}ms", start_time.elapsed().as_millis());
}
