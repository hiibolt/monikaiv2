use crate::*;

#[test]
fn initialize() -> Result<(), ()>{
    let character_json_str = std::fs::read_to_string("data/monikai.json")
        .map_err(|_| ())?;

    let monikai: monikai::Monikai = serde_json::from_str(&character_json_str)
        .map_err(|_| ())?;
    
    println!("{:?}", monikai);

    Ok(())
}

#[tokio::test]
async fn build_memory() -> Result<(), ()> {
    let conversation = "MC: Hello!\nMonika: Hi!\nMC:Do you have any good book recommendations?\nMonika: Dune - Frank Herbert!!";

    let memory = memory::Memory::new(conversation.to_string()).await;
    
    println!("{:?}", memory);
    
    Ok(())
}

#[tokio::test]
async fn build_monikai_repl() -> Result<(), ()> {
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
    tokio::spawn(monikai_repl( monikai.clone() ));

    Ok(())
}
#[tokio::test]
async fn build_monikai_backend() -> Result<(), ()> {
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

    // Start the repl and backend
    tokio::spawn(monikai_backend( monikai.clone() ));

    Ok(())
}