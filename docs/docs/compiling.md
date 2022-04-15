## Windows Pre Compile Tasks
1. Build the OpenSSL devel library through https://github.com/microsoft/vcpkg#quick-start-windows
2. Setup the MySQL-devel library
3. Recommended to setup [NVM](https://dev.to/skaytech/how-to-install-node-version-manager-nvm-for-windows-10-4nbi) currently we use the latest Node and
4. Setup [Rust](https://rustup.rs/#)
## Linux Pre Compile Tasks
1. Recommended to setup [NVM](https://github.com/nvm-sh/nvm#installing-and-updating) currently we use the latest Node and NPM
2. Setup [Rust](https://rustup.rs/#)
## Compiling
1. Pull latest code from https://github.com/wherkamp/nitro_repo.git `git clone https://github.com/wherkamp/nitro_repo.git`
### Compiling for Use
1. On Linux run the the `build.sh` file. `nitro_repo.tar.gz` file will be produced.
### Frontend Setup
1. Inside the site directory
2. Run `npm install`
3. Run `npm run build`
### Compiling Application
1. Inside the nitro_repo directory
2. run `cargo build --features ssl --release` for full version. Removing `--release` is recommended if you are doing development. `--features ssl` adds ssl support. You can remove this if you do not want the web server to use ssl

### After
1. After that you should have a nitro_repo executable either in your target/debug/ or target/release depending if you had a release tag set
## Common Issues
### Mysql Linkage Issue
1. You are honestly going to have to look this one up. Depending on your system you have to specific paths to the right files
### OpenSSL linkage issues. 
1. Really only a Windows issue but please ensure vcpkg was setup correctly
