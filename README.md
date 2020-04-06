![Rust](https://github.com/anshulrgoyal/bust/workflows/Rust/badge.svg)
# bust
College minor Project

```
Usage: ./target/release/bust <url> [-a <auth>] [-C <cookies>] [-M <method>] -c <concurrency> -n <total-request> [-H <headers>] [-f <file>] [-d <data>]

A tool for Stress Testing

Options:
  -a, --auth        pass username  and password in form of username:password
  -C, --cookies     provide cookie for the request
  -M, --method      custom http method
  -c, --concurrency concurrency the number of concurrent request
  -n, --total-request
                    total number of request made
  -H, --headers     custom header for request
  -f, --file        file path to upload the file
  -d, --data        data to be sent in request
  --help            display usage information

```