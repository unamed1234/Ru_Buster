use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (method, header) = parse_flags();
    let args: Vec<String> = env::args().collect();
    //usage help
    if args.len() < 3 {
        println!("usage is {} <example.com> <wordlist.txt>", &args[0]);
        println!("--header for custom header in \"Header: Value\" format");
        println!("-m or --method for any http method (default is get ) ");
        return Ok(());
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
        let mut body = get(
            &domain,
            &dir.replace(" ", "%20"),
            &mut stream,
            &header,
            &method,
        )
        .await?;
        // if request failed try again with new tcpstream
        if !body.starts_with("HTTP") {
            stream = TcpStream::connect(format!("{domain}:80"))?;
            body = get(
                &domain,
                &dir.replace(" ", "%20"),
                &mut stream,
                &header,
                &method,
            )
            .await?;
        }
        let status = body.split(' ').nth(1).unwrap_or("");
        if status != "404" {
            println!("found something! {} status: {}", &dir, &status);
        }
    }
    Ok(())
}
// beatifully optimized http get function (but it is ugly i know)
async fn get(
    url: &str,
    dir: &str,
    stream: &mut TcpStream,
    header: &str,
    method: &str,
) -> std::io::Result<String> {
    let mut response = String::new();
    let mut req = Vec::with_capacity(255);
    req.extend_from_slice(method.as_bytes());
    req.extend_from_slice(b" /");
    req.extend_from_slice(dir.as_bytes());
    req.extend_from_slice(b" HTTP/1.1\r\nHOST: ");
    req.extend_from_slice(url.as_bytes());
    req.extend_from_slice(b"\r\nUser-Agent: Ru_dirbuster/0.0.2\r\n");
    req.extend_from_slice(header.as_bytes());
    req.extend_from_slice(b"\r\n\r\n");
    let _ = stream.write_all(&req);
    req.clear();
    let mut line_read = BufReader::new(stream);
    let _ = line_read.read_line(&mut response);
    Ok(response)
}
fn parse_flags() -> (String, String) {
    let args: Vec<String> = env::args().collect();
    let mut a = 0;
    let mut method = String::from("GET");
    let mut header = String::from("");
    for arg in args {
        match a {
            1 => {
                header = arg;
                a = 0;
                continue;
            }
            2 => {
                method = arg;
                a = 0;
                continue;
            }
            _ => {}
        }
        match arg.as_str() {
            "--header" => a = 1,
            "-m" | "--method" => a = 2,
            _ => a = 0,
        }
    }
    (method, header)
}
