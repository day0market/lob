# Aggregated Limit Order Book GRPC server



## Start server
``cargo run --package lob --bin server -- -s "BTC/USDT"``

```
Options:
  -s, --symbol <SYMBOL>                  
  -t, --top-book-depth <TOP_BOOK_DEPTH>  [default: 10]
  -p, --port <PORT>                      [default: 50051]
  -h, --help                             Print help information
  -V, --version                          Print version information

```


## Start client
``cargo run --package lob --bin client``

```
Options:
  -p, --port <PORT>  [default: 50051]
  -h, --help         Print help information
  -V, --version      Print version information

```