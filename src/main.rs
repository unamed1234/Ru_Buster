use std::env;
use std::fs::File;
use std::io::BufRead as stdBufRead;
use std::io::BufReader as stdBufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::task;
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (method, header, args, thread_num, wordlist, domain) = parse_flags();
    //usage help
    if wordlist.is_empty() || domain.is_empty() {
        println!(
            "usage is: {} --url example.com -w wordlist.txt -m POST -H \"Authorization: 123sjdoajdoa102skda\"",
            &args[0]
        );
        println!("flags:");
        println!("-H or --header for custom header in \"Header: Value\" format");
        println!("-m or --method for any http method (default is get ) ");
        println!("-u or --url the url to target server (only HTTP is supported at this time) ");
        println!("-w or --wordlist path to your wordlist");
        return Ok(());
    }
    let filename = Arc::new(wordlist.clone());
    let met = Arc::new(method);
    let head = Arc::new(header);
    let dom = Arc::new(format!("{domain}:80"));
    let reader = stdBufReader::new(File::open(wordlist)?);
    let num_of_lines = reader.lines().count();
    let chunk = num_of_lines / thread_num;
    let remainder = num_of_lines % thread_num;
    let mut handles = Vec::new();
    for i in 0..thread_num {
        let file_clone = Arc::clone(&filename);
        let met_clone = Arc::clone(&met);
        let header_clone = Arc::clone(&head);
        let domain_clone = Arc::clone(&dom);
        if remainder == 0 {
            handles.push(task::spawn(async move {
                let line_start = chunk * i + 1;
                let read = stdBufReader::new(File::open(file_clone.to_string()).unwrap());
                let stream = TcpStream::connect(domain_clone.as_ref()).await.unwrap();
                let mut lines = read.lines();
                lines.nth(line_start);
                brute_forcer(
                    chunk,
                    &domain_clone,
                    file_clone,
                    stream,
                    &header_clone,
                    &met_clone,
                    line_start,
                )
                .await
                .unwrap()
            }));
        } else {
            handles.push(task::spawn(async move {
                let line_start = if thread_num == i {
                    chunk % i + 1
                } else if i == 0 {
                    chunk
                } else {
                    chunk * i + 1
                };
                let read = stdBufReader::new(File::open(file_clone.to_string()).unwrap());
                let stream = TcpStream::connect(domain_clone.as_ref()).await.unwrap();
                let mut lines = read.lines();
                lines.nth(line_start);
                brute_forcer(
                    chunk,
                    &domain_clone,
                    file_clone,
                    stream,
                    &header_clone,
                    &met_clone,
                    line_start,
                )
                .await
                .unwrap()
            }))
        };
    }
    for h in handles {
        h.await?;
    }
    Ok(())
}
async fn brute_forcer(
    max: usize,
    domain_clone: &str,
    file_clone: Arc<String>,
    mut stream: TcpStream,
    header_clone: &str,
    met_clone: &str,
    line_start: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let read = stdBufReader::new(File::open(file_clone.to_string()).unwrap());
    let mut lines = read.lines();
    lines.nth(line_start);
    let mut times = 0;
    for (iterations, line) in lines.enumerate() {
        if iterations >= max {
            break;
        }
        if line.as_ref().unwrap() == "index.html" {
            times += 1;
            println!("got index.html {}", times);
        }
        let mut body = http_request(
            domain_clone,
            line.as_ref().unwrap_or(&String::from("")),
            &mut stream,
            header_clone,
            met_clone,
        )
        .await
        .unwrap_or_default();
        if !body.starts_with("HTTP") {
            stream = TcpStream::connect(&domain_clone).await.unwrap();
            body = http_request(
                domain_clone,
                line.as_ref().unwrap_or(&String::from("")),
                &mut stream,
                header_clone,
                met_clone,
            )
            .await
            .unwrap_or_default();
        }
        let status = body.split(" ").nth(1).unwrap_or_default();
        if status != "404" {
            println!("found something! dir:{} status: {}", line.unwrap(), status);
        }
    }
    Ok(())
}
// beatifully optimized http http_request function (but it is very ugly i know)
async fn http_request(
    url: &str,
    dir: &str,
    stream: &mut TcpStream,
    header: &str,
    method: &str,
) -> std::io::Result<String> {
    let mut response = String::new();
    let mut request = Vec::with_capacity(255);
    // imagine this code below as let request = format!("{method} /{dir} HTTP/1.1 \r\n\r\nHOST: {url}
    // User-Agent: Ru_dirbuster/0.0.2 {header}\r\n\r\n")
    request.extend_from_slice(method.as_bytes());
    request.extend_from_slice(b" /");
    request.extend_from_slice(dir.replace(" ", "%20").as_bytes());
    request.extend_from_slice(b" HTTP/1.1\r\nHOST: ");
    request.extend_from_slice(url.as_bytes());
    request.extend_from_slice(b"\r\nUser-Agent: Ru_dirbuster/0.1.0\r\n");
    request.extend_from_slice(header.as_bytes());
    request.extend_from_slice(b"\r\n\r\n");
    let _ = stream.write_all(&request).await;
    let mut line_read = BufReader::new(stream);
    let _ = line_read.read_line(&mut response).await;
    Ok(response)
}
fn parse_flags() -> (String, String, Vec<String>, usize, String, String) {
    let args: Vec<String> = env::args().collect();
    let mut a = 0;
    let mut method = String::from("GET");
    let mut header = String::from("");
    let mut thread_num: usize = 10;
    let mut wordlist = String::from("");
    let mut url = String::from("");
    for arg in args.iter() {
        match a {
            1 => {
                header = String::from(arg);
                a = 0;
                continue;
            }
            2 => {
                method = String::from(arg);
                a = 0;
                continue;
            }
            3 => {
                thread_num = arg.parse().unwrap();
                a = 0;
                continue;
            }
            4 => {
                wordlist = String::from(arg);
                a = 0;
                continue;
            }
            5 => {
                url = String::from(arg);
                a = 0;
                continue;
            }
            _ => {}
        }
        match arg.as_str() {
            "-H" | "--header" => a = 1,
            "-m" | "--method" => a = 2,
            "-t" | "--threads" => a = 3,
            "-w" | "--wordlist" => a = 4,
            "-u" | "--url" => a = 5,
            _ => a = 0,
        }
    }
    (method, header, args, thread_num, wordlist, url)
}
