#[cfg(test)]
mod tests;
mod openai;
mod memory;
mod monikai;
mod linalg;
mod print;

use std::{
    fs::{ OpenOptions, File },
    env,
    io::Read,
    sync::{ Arc }
};
use tokio::sync::Mutex;
use serde::{ Serialize, Deserialize };


#[tokio::main]
async fn main() {
    print::info("Initializing Monikai");
    // Open the data file
    let mut character_file_handle: File = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data/monikai.json")
        .expect("Unable to get handle on './data/monikai.json'!");

    // Read from said data file
    let mut character_json_string = String::new();
    character_file_handle.read_to_string(&mut character_json_string)
        .expect("Unable to read './data/monikai.json'!");

    // Close the file handle
    drop(character_file_handle);
    print::info("Done!");

    // Build a thread and asynchronus reference to the character
    let monikai = Arc::new(
        Mutex::new(
            serde_json::from_str::<monikai::Monikai>(&character_json_string)
                .expect("Unable to parse!")
        ));

    // Start the repl and frontend
    tokio::spawn(monikai::monikai_repl( monikai.clone() ));
    tokio::spawn(monikai::monikai_backend( monikai.clone() ));

    // I may need to look into doing this a different way.
    // However, I'll probably end up using the main fn for timing tasks and plugins.
    loop {}
}