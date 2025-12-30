use std::{fs::{self, FileType}, io::Result};
fn main() -> Result<()> {

    println!("building");
    let mut protos = Vec::new();
    let filelist = fs::read_dir("../evotypes/").unwrap();

    for i in filelist {
        let v = i.unwrap();
        println!("{:?}", v.path().extension().unwrap());
        if v.path().extension().unwrap() == "proto" {
            println!("{:?}", v.path().clone());
            protos.push(v.path());
        }
    }
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(Name)]");

    config.compile_protos(&protos, &["../evotypes/"])?;
    Ok(())
}
