use colorama::Colored;

pub fn info( input: &str ) {
    println!("{}", String::from(input).color("blue") );
}

pub fn monikai( input: &str ) {
    println!(">{}", String::from(input).color("bright magenta") );
}

pub fn debug( input: &str ) {
    println!("{}", format!("[{}]", input).color("green") );
}