// #[macro_use]
extern crate lazy_static;

extern crate winapi;

use std::io::Result;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fmt::Error, fs::OpenOptions};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::{
    Win32::Foundation::PWSTR,
    Win32::UI::Shell::{ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems},
};

pub fn browse_to_file(filename: &str)
{
    println!("Browsing to file: {}", filename);

    let wide: Vec<u16> = OsStr::new(filename)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();

    unsafe {
        let pidl = ILCreateFromPathW(PWSTR(wide.as_ptr() as *mut u16));
        if !pidl.is_null()
        {
            let result = SHOpenFolderAndSelectItems(pidl, 0, std::ptr::null(), 0);
            match result
            {
                Ok(_) => println!("Successfully opened the folder and selected the item."),
                Err(e) => println!(
                    "Failed to open the folder and select the item. -> {:?}",
                    e.to_string()
                ),
            }
            ILFree(pidl);
        }
    }
}

pub fn sort_path_vectors(vectors: &mut [&mut Vec<walkdir::DirEntry>])
{
    for vector in vectors.iter_mut()
    {
        vector.sort_by_key(|entry| entry.path().to_owned());
    }
}

lazy_static! {
    static ref PRINT_LOCK: Mutex<()> = Mutex::new(());
    static ref ARGV0: String = std::env::args().by_ref().take(1).collect();
}

pub fn synchronized_println(args: std::fmt::Arguments)
{
    let _lock = PRINT_LOCK.lock().unwrap();
    println!("{}", args);
}
pub fn synchronized_println_with_string(text: String)
{
    let _lock = PRINT_LOCK.lock().unwrap();
    println!("{}", text);
}

pub fn synchronized_eprintln(args: std::fmt::Arguments)
{
    let _lock = PRINT_LOCK.lock().unwrap();
    eprintln!("{}", args);
}

pub fn synchronized_eprintln_with_string(text: String)
{
    let _lock = PRINT_LOCK.lock().unwrap();
    eprintln!("{}", text);
}

pub fn wait_for_user_continue()
{
    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).expect("Failed to read line");
}

pub fn clear_screen()
{
    // println!("\x1B[2J\x1B[1;1H");
    // crossterm::execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
    clearscreen::clear().unwrap();
}

pub fn get_log_file() -> Result<PathBuf>
{
    let mut path = PathBuf::from("");
    let mut args = std::env::args();

    if let Some(first_arg) = args.nth(1)
    {
        println!("The first argument is: {}", &first_arg);
        path = PathBuf::from(first_arg);
        path.parent().unwrap();
        path.set_file_name("skipped_files.txt")
    }
    else
    {
        panic!("Argv0 was not passed. Closing...")
    }

    Ok(path)
}

pub fn clear_log_file()
{
    let mut path = PathBuf::from("");
    let mut args = std::env::args();

    if let Some(first_arg) = args.nth(0)
    {
        println!("The first argument is: {}", &first_arg);
        path = PathBuf::from(first_arg);
        path.parent().unwrap();
        path.set_file_name("skipped_files.txt")
    }
    else
    {
        println!("Argv0 was not passed. Closing...");
        std::thread::sleep(std::time::Duration::from_secs(5));
        std::process::exit(1);
    }

    match OpenOptions::new().write(true).truncate(true).open(&path)
    {
        Ok(_) => synchronized_println(format_args!(
            "Successfully cleared the log file: {:?}",
            &path
        )),
        Err(_) => synchronized_eprintln(format_args!("Failed to clear the log file: {:?}", &path)),
    };
}
