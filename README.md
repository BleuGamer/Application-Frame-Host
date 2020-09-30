# What is this?

Application Frame Host is an application admin interface for use in servers or
other remote use cases written in Rust and Angular.js. If you're running a game
server (i.e. Factorio), youc ould go into the webserver where this is hosted and
apply commands and logic to your application to manipulate it, start/stop it,
access logs or system resources, etc.

This has potential enterprise use as well as I plan to add in distributed
command structures for multiple remote applications.

The admin software itself will utilize generic `REST` apis as well as `websocket`
connectors so any frontend application will be able to pull data.

This is an early work in progess for my own needs, but anyone is welcome to
contribute and use the software.

## Current Build Steps:

### Versions: 
Node.js: **v12.18.4**
Rustc: **1.46.0**

### Frontend Development:

npm install &&
npm run build &&
npm run start

### With Rust:

npm install &&
./build.sh build
