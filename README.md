# tcad
tcloud automatic downloader

https://github.com/Lunik/tcloud-heroku

## Dependencies
Install the following:
- [wget](https://chocolatey.org/packages/Wget) (comes pre-installed in Linux on most distros)
For Windows Users: `choco install wget`

## Installation
## Windows
Download [release](https://github.com/sathwikmatsa/tcad/releases/download/0.2/TCAD_For_Windows.zip)

Extract it and run `install.bat`

## Linux
Download the readily available binary at https://github.com/sathwikmatsa/tcad/releases/download/0.1/tcad and skip the `Build` section.

## Build
```
> git clone https://github.com/sathwikmatsa/tcad.git
> cd tcad
> cargo build --release
```

## Configure
[`> cp .env_sample .env`](https://github.com/sathwikmatsa/tcad/blob/master/.env_sample)

**substitue** dummy values in `.env` with actuals

## Setup
After `cargo build`:
```
> mkdir -p ~/.bin/tcad
> cp target/release/tcad ~/.bin/tcad # if you directly downloaded binary, use: cp tcad ~/.bin/tcad
> cp .env ~/.bin/tcad
```
Add a cron job
```
> crontab -e
```
append the following line* :
```
*/5 * * * * sudo -u <user> DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/<uid>/bus /home/<user>/.bin/tcad/tcad /home/<user>/.bin/tcad/.env >> /home/<user>/.bin/tcad/tcad.log
```
to run tcad every `5 minutes`.

*Note: replace \<user\> and \<uid\> in the above line. (use `whoami` to get user, `id -u $(whoami)` to get user id)
