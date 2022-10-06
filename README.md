Here I'll be posting DirectX/WinAPI programming stuff (all examples are written
in Rust using `windows` crate).

To cross-compile from MacOS and run under `wine`
```
brew install mingw-w64
brew tap gcenx/wine
brew install wine-crossover
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
wine target/x86_64-pc-windows-gnu/debug/project.exe
```
