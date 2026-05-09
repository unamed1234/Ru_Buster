use rustls_pki_types::ServerName;
use std::env;
use std::fs::File;
use std::io::BufRead as stdBufRead;
use std::io::BufReader as stdBufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::task;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_args = parse_flags();
    //usage help
    if http_args.wordlist.is_empty() || http_args.domain.is_empty() {
        println!(
            "usage is: {} --url example.com -w wordlist.txt -m POST -H \"Authorization: 123sjdoajdoa102skda\"",
            &http_args.args[0]
        );
        println!("flags:");
        println!("-H or --header for custom header in \"Header: Value\" format");
        println!("-m or --method for any http method (default is get ) ");
        println!("-u or --url the url to target server (only HTTP is supported at this time) ");
        println!("-w or --wordlist path to your wordlist");
        return Ok(());
    }
    let filename: Arc<str> = Arc::from(http_args.wordlist.as_str());
    let met = Arc::new(http_args.method);
    let head = Arc::new(http_args.header);
    let rightdom = if http_args.domain.starts_with("https://") {
        Arc::new(format!(
            "{}:443",
            http_args.domain.strip_prefix("https://").unwrap()
        ))
    } else if http_args.domain.starts_with("http://") {
        Arc::new(format!(
            "{}:80",
            http_args.domain.strip_prefix("http://").unwrap()
        ))
    } else {
        Err("make sure your domain starts with a protocol (https:// or http://)")?
    };
    let dom = Arc::new(http_args.domain);
    let reader = stdBufReader::new(File::open(&http_args.wordlist)?);
    let num_of_lines = reader.lines().count();
    let chunk = num_of_lines / http_args.thread_num;
    let remainder = num_of_lines % http_args.thread_num;
    let mut handles = Vec::new();
    for i in 0..http_args.thread_num {
        let file_clone = Arc::clone(&filename);
        let met_clone = Arc::clone(&met);
        let header_clone = Arc::clone(&head);
        let domain_clone = Arc::clone(&dom);
        // let http_domain_clone = Arc::clone(&http_dom);
        let right_domain_clone = Arc::clone(&rightdom);
        // let https_domain_clone = Arc::clone(&http_dom);
        if domain_clone.starts_with("http://") {
            handles.push(task::spawn(async move {
                let line_start = chunk * i;
                let thread_chunk = if i == http_args.thread_num - 1 {
                    chunk + remainder
                } else {
                    chunk
                };
                let stream = TcpStream::connect(right_domain_clone.as_ref())
                    .await
                    .unwrap();
                http_brute_forcer(
                    thread_chunk,
                    &right_domain_clone,
                    file_clone,
                    stream,
                    &header_clone,
                    &met_clone,
                    line_start,
                )
                .await
                .unwrap()
            }))
        } else if domain_clone.starts_with("https://") {
            handles.push(task::spawn(async move {
                let line_start = chunk * i;
                let thread_chunk = if i == http_args.thread_num - 1 {
                    chunk + remainder
                } else {
                    chunk
                };
                https_brute_forcer(
                    thread_chunk,
                    &right_domain_clone,
                    file_clone,
                    &header_clone,
                    &met_clone,
                    line_start,
                )
                .await
                .unwrap()
            }))
        }
    }
    for h in handles {
        h.await?;
    }
    Ok(())
}
async fn http_brute_forcer(
    max: usize,
    domain: &str,
    file_clone: Arc<str>,
    mut stream: TcpStream,
    header_clone: &str,
    met_clone: &str,
    line_start: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let read = stdBufReader::new(File::open(&*file_clone).unwrap());
    let mut lines = read.lines();
    if line_start > 0 {
        lines.nth(line_start - 1);
    }
    for (iterations, line) in lines.enumerate() {
        if iterations >= max {
            break;
        }
        let mut body = http_request(
            domain,
            line.as_ref().unwrap_or(&String::from("")),
            &mut stream,
            header_clone,
            met_clone,
        )
        .await
        .unwrap_or_default();
        if !body.starts_with("HTTP") {
            stream = TcpStream::connect(&domain).await?;
            body = http_request(
                domain,
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
            // fix next request being body to this one since we're reusing stream
            stream = TcpStream::connect(&domain).await?;
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
async fn https_brute_forcer(
    max: usize,
    domain: &str,
    file_clone: Arc<str>,
    header_clone: &str,
    met_clone: &str,
    line_start: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let read = stdBufReader::new(File::open(&*file_clone).unwrap());
    let mut lines = read.lines();
    if line_start > 0 {
        lines.nth(line_start - 1);
    }
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let addr = domain
        .strip_suffix(":443")
        .expect("something went very wrong hardcoded data doesn't exist?");
    let connector = TlsConnector::from(Arc::new(config));
    let dnsname: ServerName<'static> = ServerName::try_from(addr).unwrap().to_owned();
    let stream = TcpStream::connect(domain).await?;
    let mut stream = connector.connect(dnsname.clone(), stream).await?;
    for (iterations, line) in lines.enumerate() {
        if iterations >= max {
            break;
        }
        let dir = line?;
        let mut body = https_request(&mut stream, addr, &dir, met_clone, header_clone)
            .await
            .unwrap_or_default();
        if !body.starts_with("HTTP") {
            stream = tls_reconnect(domain, dnsname.clone(), &connector).await?;
            body = https_request(&mut stream, addr, &dir, met_clone, header_clone)
                .await
                .unwrap_or_default()
        }
        let status = body.split(" ").nth(1).unwrap_or_default();
        if status != "404" {
            println!("found something! dir:{} status: {}", &dir, status);
            stream = tls_reconnect(domain, dnsname.clone(), &connector).await?
        }
    }
    Ok(())
}
async fn https_request(
    stream: &mut TlsStream<TcpStream>,
    addr: &str,
    dir: &str,
    method: &str,
    header: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut request = Vec::with_capacity(255);
    request.extend_from_slice(method.as_bytes());
    request.extend_from_slice(b" /");
    request.extend_from_slice(dir.replace(' ', "%20").as_bytes());
    request.extend_from_slice(b" HTTP/1.1 \r\nHost: ");
    request.extend_from_slice(addr.as_bytes());
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(header.as_bytes());
    request.extend_from_slice(b"\r\n\r\n");
    let _ = stream.write_all(&request).await;
    let mut resp = String::new();
    let _ = stream.read_line(&mut resp).await;
    Ok(resp)
}
async fn tls_reconnect(
    domain: &str,
    dnsname: ServerName<'static>,
    connector: &TlsConnector,
) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let tcp_stream = TcpStream::connect(domain).await?;
    let stream = connector.connect(dnsname, tcp_stream).await?;
    Ok(stream)
}
fn parse_flags() -> HttpArgs {
    let args: Vec<String> = env::args().collect();
    let mut a = 0;
    let mut method = String::from("GET");
    let mut header = String::from("");
    let mut thread_num: usize = 10;
    let mut wordlist = String::from("");
    let mut domain = String::from("");
    for arg in args.iter() {
        match a {
            1 => {
                header = arg.parse().expect("header is malformed");
                a = 0;
                continue;
            }
            2 => {
                method = arg.parse().expect("method is wrong or malformed");
                a = 0;
                continue;
            }
            3 => {
                thread_num = arg.parse().expect("threads is wrong or malformed");
                a = 0;
                continue;
            }
            4 => {
                wordlist = arg.parse().expect("wordlist is wrong or malformed");
                a = 0;
                continue;
            }
            5 => {
                domain = arg.parse().expect("domain is wrong or malformed");
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
    HttpArgs {
        method,
        header,
        wordlist,
        domain,
        args,
        thread_num,
    }
}
struct HttpArgs {
    method: String,
    header: String,
    wordlist: String,
    domain: String,
    args: Vec<String>,
    thread_num: usize,
}
