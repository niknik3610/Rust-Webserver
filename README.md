# Rust-Webserver

A multi-threaded web server & client built in Rust, using the Socket-2 api. Uses port#-8000 for server. Sig-int quits after receiving a package (even if there is no package being processed). This is to prevent the server from quitting during a request. 
