use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get args 
    let args: Vec<String> = env::args().collect();
    //usage help
    if args.len() != 3 {
        println!("usage is {} <example.com> <wordlist.txt> ",&args[0]);
        process::exit(1);
    }
    //open wordlist
    let domain = args[1].clone();
    let file = File::open(args[2].clone())?;
    let reader = BufReader::new(file);
    // initial tcp connection
    let mut stream = TcpStream::connect(format!("{domain}:80"))?;
    // make get request iterating trough each dir in wordlist
    for line in reader.lines() {
        let dir = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let mut body = get(&domain, &dir,&mut stream).await?;
        // if request failed try again with new tcpstream
        if ! body.starts_with("HTTP"){
            stream = TcpStream::connect(format!("{domain}:80"))?;
            body = get(&domain, &dir,&mut stream).await?;
        }
        let status = body.split(' ').nth(1).unwrap_or("");
        if status != "404" {
            println!("found something! {} status: {}", &dir, &status);
        }
    } 
    Ok(())
}
// beatifully optimized http get function 
async fn get(url: &str,dir: &str,stream: &mut TcpStream) -> std::io::Result<String> {
    let mut response = String::new();
    let mut req = Vec::with_capacity(256);
    req.extend_from_slice(b"GET /");
    req.extend_from_slice(dir.as_bytes());
    req.extend_from_slice(b" HTTP/1.1\r\nHOST: ");
    req.extend_from_slice(url.as_bytes());
    req.extend_from_slice(b"\r\nUser-Agent: Ru_dirbuster/0.0.1\r\n\r\n");
    let _ = stream.write_all(&req);
    req.clear();
    let mut line_read = BufReader::new(stream);
    let _ = line_read.read_line(&mut response);
    Ok(response)
}
