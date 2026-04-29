# Ru_Buster

web directory brute forcer written 100% in rust

# features

supports threading using tokio runtime which is very fast and now supports https with custom optimized https function

# usage

it supports headers trough -H flag in Header: value format, and any method trough the -m  flag (literally any method it doesnt check if its valid)
run without args for usage
usage is: Ru_Buster --url example.com -w wordlist.txt -m POST -H "Authorization: 123sjdoajdoa102skda"
flags:
-H or --header for custom header in \Header: Value\ format
-m or --method for any http method (default is get )
-u or --url the url to target server (only HTTP is supported at this time)
-w or --wordlist path to your wordlist

# fastest?

maybe? I've only tested against gobuster its not as mature as gobuster but this has very fast threading which from my tests (keyword MY tests) is way faster than gobuster, have fun!
