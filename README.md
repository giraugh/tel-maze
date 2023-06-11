# tel-maze

A raw TCP server that allows users to explore a simple maze environment.

## Building

`tel-maze` requires a rust installation including the `cargo` cli tool.
There is also a Dockerfile if preferred however.

First clone the repo
```
git clone https://github.com/giraugh/tel-maze
cd tel-maze
```

Then build and run it with cargo
```
cargo run
```

Users can now connect to it using a tpc client like netcat (`nc`)
```
nc localhost 5000
```

## Contributing

Any and all contributes are welcomed :)
