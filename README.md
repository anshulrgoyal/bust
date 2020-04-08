![Rust](https://github.com/anshulrgoyal/bust/workflows/Rust/badge.svg)

# bust
It is a simple server bench marking tool it is not a scientific tool for bench marking it provide a very crude idea of the load management capability of your server.

# Highlights
- It support both **HTTP** and **HTTPS** .
- Custom method can be added to request using `-M` option .
- Any header can be added to request using `-H` option and repeation is allowed .
- File upload is also supported throught `-f` flag with mutlipart/formdata content-type header.
- Body can be passed to supported type of request using `-d` flag .
- Number of concurrent request is required. Can be passed with `-c` option.
- Total number of request should also be passed using `-n` flag.
- Auth details can be passed using `-a` option.

```
Usage: bust <url> [-a <auth>] [-C <cookies>] [-M <method>] -c <concurrency> -n <total-request> [-H <headers>] [-f <file>] [-d <data>]

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

## Add Header to Request
Adding a header is simple as adding just a option with format of `<header_name>=<header_value>` for example `content-type=application/json` . We can add any number of headers.

```bash

$ bust -n 20 -c 5 https://www.google.com -H auth=<auth-token> -H user-agent=<user-agent-name>

``` 

## Add Custom Method for Request
Adding custom method is done by using `-M` option. All the **HTTP** method are supported. Eg. _POST_ , _PUT_ etc.

```bash

$ bust -n 20 -c 5 https://www.google.com -M POST -H auth=<auth-token> -H user-agent=<user-agent-name>

``` 

## Add file for upload
Upload file with request using multipart/formdata header. Eg. `-f <field-name>=<file-path>` .

```bash

$ bust -n 20 -c 5 https://www.google.com -M POST -f image=./path/to/file

``` 
where `image` is field-name and `./path/to/file` is path of file.

## Adding Body to request
Body is passed using `-d` in form of string. Eg . -d \{\"name\":\"bust\"\}

```bash

$ bust -n 20 -c 5 https://www.google.com -M POST -H content-type=application/json -d \{\"name\":\"bust\"\}

``` 

## Adding Cookies to request
Adding cookie is simple using `-C` option which is repeatable. Syntax `-C <cookie>` .

```bash

$ bust -n 20 -c 5 https://www.google.com -M POST -C auth=76rtitutuit

``` 