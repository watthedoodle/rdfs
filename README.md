# RDFS

🔧 Rust Distributed File System (RDFS) a toy implementation of the Google File System (GFS)

![alt Experimental](https://img.shields.io/badge/Type-Experimental-red.svg)
![alt Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![alt Binary](https://img.shields.io/badge/Binary-Polymorphic-green.svg)

```shell

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

## Simulating a cluster using docker

In order to test our distributed cluster, instead of spinning up lots of heavy Virtual Machines, instead we can "simulate" it using lightweight containers.

First we will need to build our container images via the following commnand:

```shell
$ docker compose build
```

This will take some time but eventually once it completes we should have a custom docker image, we can check by doing the following:

```shell
$ docker images

REPOSITORY                                             TAG                 IMAGE ID       CREATED          SIZE
rdfs                                                   latest              e9d6e2275c17   35 minutes ago   13.2MB
```

now we can spin up our inital "cluster" with only 1 master node and 1 worker node:

```shell
$ docker compose up -d

[+] Running 3/3
 ✔ Network rdfs_default     Created                                                                                 0.1s
 ✔ Container rdfs-master-1  Started                                                                                 0.4s
 ✔ Container rdfs-worker-1  Started                                                                                 0.3s
```

Now we can _scale_ the number of worker node simply by using the `scale` command, for example if we wanted to scale up to have 3 worker nodes:

```shell
$ docker compose scale worker=3

[+] Running 3/3
 ✔ Container rdfs-worker-1  Running                                                                                 0.0s
 ✔ Container rdfs-worker-3  Started                                                                                 0.6s
 ✔ Container rdfs-worker-2  Started                                                                                 0.3s
```

If we wish to check out the logs we can do this by using the container names e.g:

```shell
$ docker logs -f rdfs-master-1

██████  ██████  ███████ ███████
██   ██ ██   ██ ██      ██
██████  ██   ██ █████   ███████
██   ██ ██   ██ ██           ██
██   ██ ██████  ██      ███████

 a toy distributed file system

==> launching node in [master] mode on port 8888...
==> got a heartbeat from worker node -> ...172.18.0.3:43640
```

Finally we can "tear down" our cluster simply by doing the following:

```shell
$ docker compose down
[+] Running 5/5
 ✔ Container rdfs-master-1  Removed                                                                                10.3s
 ✔ Container rdfs-worker-1  Removed                                                                                10.3s
 ✔ Container rdfs-worker-2  Removed                                                                                10.2s
 ✔ Container rdfs-worker-3  Removed                                                                                10.3s
 ✔ Network rdfs_default     Removed                                                                                 0.1s
```
