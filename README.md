# Project::Tarot

Simple implementation of Long-Short Term Memory (LSTM) neural layer with Graphical User Interface
(GUI).

![Screenshot of Project::Tarot](resource/screenshot.png)

> DISCLAIMER: Purpose of this app is to demonstrate author's learning process. This app CANNOT
> guarantee prediction accuracy nor provide any warranty. Do it on your own risk.
>
> For professional usage, email contact@athaariq.my.id.

## Acknowledgement

The app is possible to be developed thanks to these projects:

-   [Tauri][1]: Cross-platform app development framework
-   [Burn Framework][2]: Cross-platform AI crate for Rust programming language
-   [Material UI][3]: User Interface (UI) framework, based on ReactJS
-   [NdArray][4]: N-dimensional array library
-   [OpenBLAS][5]: Optimized Basic Linear Algebra Subprogram (BLAS)
-   [SVGRepo][7]: Free vector icons in SVG format

...and Other packages mentioned in `package.json` and crates in `src-tauri/Cargo.toml`!

[1]: https://v2.tauri.app/
[2]: https://burn.dev/
[3]: https://mui.com/
[4]: https://docs.rs/ndarray/latest/ndarray/
[5]: https://github.com/OpenMathLib/OpenBLAS#readme
[7]: https://www.svgrepo.com/

## How to use

Either download in [release page][8], or build source code in your own computer. After
installed, run the app and choose one of CSV files in `sample_data` folder.

[8]: https://github.com/Thor-x86/project-tarot/releases

## Building the source code

If you are interested to build the source code, then this section is right for you.

### One-time project preparation

1. Install the required software:
    - [NodeJS](https://nodejs.org/en/download/current)
    - [PNPM](https://pnpm.io/installation)
    - [Rust](https://www.rust-lang.org/tools/install)
    - [OpenBLAS](http://www.openmathlib.org/OpenBLAS/docs/install/)
2. Download this repo. Recommended to `git clone` via terminal/command-line instead:
    ```sh
    git clone https://github.com/Thor-x86/project-tarot.git
    ```
3. Open with your terminal or command prompt, then run:
    ```sh
    pnpm install
    cd src-tauri
    cargo fetch
    cd ..
    ```
4. If you are using **Linux** and will compile for Windows target, install [LLVM][6] then run this
   command:
    ```sh
    rustup target add x86_64-pc-windows-msvc
    cargo install cargo-xwin
    ```

[6]: https://releases.llvm.org/download.html

### Development mode

For experimenting the source code, you can run the program while activating auto-build feature by
using this command.

```sh
pnpm run tauri dev
```

### Deployment

To package the program, normally you can just run this command.

```sh
pnpm run tauri build
```

If you are using Linux and want to package the app for Windows, run this instead.

```sh
pnpm run tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
```

## Project status

The author does not expect this project will be massively used. So, the maintenance only limited to
bug fixes. However, it can be considered to be active and grow a community over it if there is a
strong demand from open source community in general.

## License

Project::Tarot is distributed under the terms of GPL-3.0 license. See [LICENSE](LICENSE) for
details. Opening a pull request is assumed to signal agreement with these licensing terms.
