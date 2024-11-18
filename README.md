<div align="center">

<pre>
 ██████╗ ██╗   ██╗ █████╗ ██████╗  █████╗ ███╗   ██╗████████╗██╗███╗   ██╗███████╗
██╔═══██╗██║   ██║██╔══██╗██╔══██╗██╔══██╗████╗  ██║╚══██╔══╝██║████╗  ██║██╔════╝
██║   ██║██║   ██║███████║██████╔╝███████║██╔██╗ ██║   ██║   ██║██╔██╗ ██║█████╗  
██║▄▄ ██║██║   ██║██╔══██║██╔══██╗██╔══██║██║╚██╗██║   ██║   ██║██║╚██╗██║██╔══╝  
╚██████╔╝╚██████╔╝██║  ██║██║  ██║██║  ██║██║ ╚████║   ██║   ██║██║ ╚████║███████╗
 ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝  ╚═══╝╚══════╝
----------------------------------------------------------------------------------
quickly and easily create sandbox to run untrusted code. Made with ❤️ using 🦀
</pre>

[![Crates.io](https://img.shields.io/crates/v/quarantine.svg)](https://crates.io/crates/quarantine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

> `quarantine` is a command-line tool that quickly and easily gives you access to a sandboxed environment to run untrusted code.
It takes your current working directory, mounts it inside a docker container, and provides a shell interface.

## Installation

### Arch Linux

[quarantine](https://aur.archlinux.org/packages/quarantine) is available as a package in the [AUR](https://aur.archlinux.org).<br>
You can install it with your preferred [AUR helper](https://wiki.archlinux.org/title/AUR_helpers). example:

```sh
paru -S quarantine
```

### Other

[Download the binary](https://github.com/zahash/quarantine/releases)

( or )

```
cargo install quarantine
```

## Usage examples

```sh
quarantine --help
quarantine -i node:latest
```

## Meta

zahash – zahash.z@gmail.com

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/zahash/](https://github.com/zahash/)

## Contributing

1. Fork it (<https://github.com/zahash/quarantine/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request
