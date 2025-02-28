### Purego

To execute the purego example we have to build it and pass the correct linker flags so it finds the dynamic libraries 

```
go build -ldflags="-linkmode external -extldflags '-Wl,-rpath,@executable_path/rustlib/target/release -Wl,-rpath,@executable_path/golib'" purego/main.go
```

The build the shared libraries 

```
cd golib &&  go build -o libresult.so -buildmode=c-shared *.go
```

```
cd rustlib && cargo build --release
```

Then execute the program:

```
$ ./main libresult.so
Len: 131986552864800
Result: Hello from Go shared library!
```

```
$ ./main librustlib.dylib
Len: 0
Result: Hello from Rust shared library!
```

### CGO

We need to build the shared libraries as well:


```
cd golib &&  go build -o libresult.so -buildmode=c-shared *.go
```

```
cd rustlib && cargo build --release
```


```
$ go run cgo/main.go rustlib/target/release/librustlib.dylib
Result: Hello from Rust shared library!
Len: 31
```

```
$ go run cgo/main.go golib/libresult.so
Result: Hello from Go shared library!
Len: 30
```