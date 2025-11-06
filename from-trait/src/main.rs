use std::io;

#[derive(Debug)]
enum MyError{
    Io(io::Error),
    Parse(std::num::ParseIntError),
}

impl From<io::Error> for MyError{
    fn from(error: io::Error) -> Self{
        MyError::Io(error)
    }
}
impl From<std::num::ParseIntError> for MyError{
    fn from(error: std::num::ParseIntError) -> Self{
        MyError::Parse(error)
    }
}

fn main()->Result<(),MyError>{
    let content = std::fs::read_to_string("main.c")?;
    let _num:i32=content.trim().parse()?;
    Ok(())
}