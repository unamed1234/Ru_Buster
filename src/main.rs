use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("usage is {} <example.com> <wordlist.txt> ",&args[0]);
        process::exit(1);
    }
    let domain = args[1].clone();
    let file = File::open(args[2].clone())?;
    let reader = BufReader::new(file);
    let mut stream = TcpStream::connect(format!("{domain}:80"))?;
    for dir in reader.lines() {
        //println!("trying {dir:?}");
        //println!("{domain}");
        let body = get(&domain, dir.as_ref().expect("no dir").to_string(),&mut stream).await?;
        let status: i32 = body.split_whitespace().nth(1).unwrap_or("").parse().unwrap_or(0);
        //println!("{}",&status);
        if status == 200 {
            println!("found url! {:?}", &dir)
        }
        //println!("{:?}", body);
        if body.is_empty(){
            stream = TcpStream::connect(format!("{domain}:80"))?;
            let body = get(&domain, dir.as_ref().expect("no dir").to_string(),&mut stream).await?;
            let status: i32 = body.split_whitespace().nth(1).unwrap_or("").parse().unwrap_or(0);
            if status == 200 {
                println!("found url! {:?}", &dir)
        }
        }
    } 
    Ok(())
}

async fn get(url: &str,dir: String ,stream: &mut TcpStream) -> std::io::Result<String> {
    let mut response = String::new();
    stream.write_all(
        format!(
            "GET /{dir} HTTP/1.1\r\nHost: {url}\r\nUser-Agent: ruCurl/0.0.1\r\n\r\n",
        )
        .as_bytes(),
    )?;
    let mut line_read = BufReader::new(stream);
    let _ = line_read.read_line(&mut response);
    Ok(response)
}
//let mut co
