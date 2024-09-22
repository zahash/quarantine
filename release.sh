set -e

mkdir -p dist

# linux
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu

cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target i686-unknown-linux-gnu

cp target/x86_64-unknown-linux-gnu/release/quarantine dist/quarantine-x86_64-linux
cp target/aarch64-unknown-linux-gnu/release/quarantine dist/quarantine-aarch64-linux
cp target/i686-unknown-linux-gnu/release/quarantine dist/quarantine-i686-linux

# windows
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu
# rustup target add aarch64-pc-windows-msvc

cross build --release --target x86_64-pc-windows-gnu
cross build --release --target i686-pc-windows-gnu
# cross build --release --target aarch64-pc-windows-msvc

cp target/x86_64-pc-windows-gnu/release/quarantine.exe dist/quarantine-x86_64-windows.exe
cp target/i686-pc-windows-gnu/release/quarantine.exe dist/quarantine-i686-windows.exe
# cp target/aarch64-pc-windows-msvc/release/quarantine.exe dist/quarantine-aarch64-windows.exe

# apple
# rustup target add aarch64-apple-darwin
# rustup target add x86_64-apple-darwin

# cross build --release --target aarch64-apple-darwin
# cross build --release --target x86_64-apple-darwin
