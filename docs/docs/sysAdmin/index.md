# How to setup Nitro_Repo
## Pre Install Tasks
1. Install MySQL. For more information click [here](https://nitro-repo.kingtux.dev/knowledge/InternalWorkings.html#users).
2. Create a database. For nitro_repo to use
## Getting your build
Please use one of the following options for your build
1. Latest [Release](https://github.com/wherkamp/nitro_repo/releases) on Github
2. Latest [Build](https://github.com/wherkamp/nitro_repo/actions/workflows/push.yml) on Github
3. Build yourself. Instructions are [here](https://nitro-repo.kingtux.dev/compiling.html)

## Setup
1. Decompress the build inside your install directory. I use `/opt/nitro_repo`. Using the command `tar -xf nitro_repo.tar.gz` Note: You might have to decompress the zip for Github Latest Builds
2. Run `./nitro_repo --install` Follow the CLI for installation. 
3. After completing the installation go ahead and run ./nitro_repo again. To ensure proper setup. Connect to it over the browser. Using your host and port set
4. Edit other/nitro_repo.service to use the appropriate location of your installation. Then copy the nitro_repo.service to the service directory Command: `cp other/nitro_repo.service /etc/systemd/system/nitro_repo.service`
5. Run `systemctl daemon-reload` and `systemctl start nitro_repo.service`
### SSL
After installation you can add SSL

Edit cfg/nitro_repo.toml

Under the application section

Add

```toml
ssl_private_key=
ssl_cert_key=
```

Make sure to specify values

#### For Lets Encrypt 

```toml
ssl_private_key='/etc/letsencrypt/live/{domain}/privkey.pem'
ssl_cert_key='/etc/letsencrypt/live/{domain}/cert.pem'
```
### 

Finally Restart Nitro Repo