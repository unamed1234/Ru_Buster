# Ru_Buster
basic web directory brute forcer written 100% in rust 
## why?
yes (no you should probably stick to gobuster) .
## faster?
barely. trough my testing this is about 1-0.3 seconds faster than single threaded gobuster on a 4750 line wordlist attacking a nginx container in localhost (very biased test) 
but if you add threading gobuster wins due to threading not being implemented yet you shouldn't really use this unless you have a very specific need for it
