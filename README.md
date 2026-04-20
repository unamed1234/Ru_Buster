# Ru_Buster
web directory brute forcer written 100% in rust 
# threading 
now supports threading using tokio runtime and is very fast 
# usage
it supports headers trough -H flag in Header: value format, and any method trough the -m  flag 
run without args for usage 
usage is: {} --url example.com -w wordlist.txt -m POST -H Authorization: 123sjdoajdoa102skda
flags:);
-H or --header for custom header in \Header: Value\ format
-m or --method for any http method (default is get )
-u or --url the url to target server (only HTTP is supported at this time)
-w or --wordlist path to your wordlist
# fastest?
maybe? I've only tested against gobuster its not as mature as gobuster but has very fast threading which from my tests is way faster than gobuster, have fun!
