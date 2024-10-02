# RDFS

🔧 Rust Distributed File System (RDFS) a toy implementation of the Google File System (GFS)

![alt Experimental](https://img.shields.io/badge/Type-Experimental-red.svg)
![alt Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![alt Binary](https://img.shields.io/badge/Binary-Polymorphic-green.svg)

```console

██████  ██████  ███████ ███████
██   ██ ██   ██ ██      ██
██████  ██   ██ █████   ███████
██   ██ ██   ██ ██           ██
██   ██ ██████  ██      ███████

 a toy distributed file system
```

## Intro

Reading the original paper ["The Google File System"](https://pdos.csail.mit.edu/6.824/papers/gfs.pdf)
was the inspiration for HDFS _(Hadoop Distributed File System)_ that later gave way to Amazon's "S3" which has
become almost the "defacto" standard. Distributed file systems are super interesting and this project is
an attempt to understand how distributed file systems work by building a toy version of the original GFS.

![img](GFS.png)

I really like that idea of creating a single "polymorphic binary" that can act as the following:

- Master node
- Worker node
- Client CLI

## Environment Variables

This binary assumes that the following environemnt variables are present in order to setup the
required global configuration:

| Name          | Example value                        | Description                                |
| ------------- | ------------------------------------ | ------------------------------------------ |
| RDFS_ENDPOINT | https://master-node-ip:8888          | where the master node can be reached       |
| RDFS_TOKEN    | 7687a5ac-ed5a-4d69-8cc3-f78c119b3219 | the security token needed for this cluster |

## Usage: WARNING unstable will probably change

```shell
rdfs 0.1.0
Wat The Doodle <watthedoodle@gmail.com>


██████  ██████  ███████ ███████
██   ██ ██   ██ ██      ██
██████  ██   ██ █████   ███████
██   ██ ██   ██ ██           ██
██   ██ ██████  ██      ███████

 a toy distributed file system


Usage: rdfs [COMMAND]

Commands:
  list    List all remote files e.g rdfs list
  get     Get a remote file e.g rdfs get foo.txt
  add     Add a remote file e.g rdfs add foo.txt
  remove  Remove a remote file e.g rdfs remove foo.txt
  mode    Mode: run the binary in either as a "Master" or "Worker" node
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

## Authentication

For all the HTTP calls we need to pass the token as a custom header value i.e. `x-rdfs-token`. This
will be checked using an authentication middleware in axum.

## Test Harness

Some of the local tests require us to call the worker or master http endpoints, we have a folder called `test-harness` that contains those tests. To run the test execute the following:

```shell
$ deno task test
```
