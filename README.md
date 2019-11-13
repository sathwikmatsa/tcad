# tcad
tcloud automatic downloader [For Linux]

https://github.com/Lunik/tcloud-heroku

## Dependencies
- [Rust](https://www.rust-lang.org/tools/install) (if you want to build locally)
- [aria2c](https://aria2.github.io/)
- [notify-send](https://www.archlinux.org/packages/extra/x86_64/libnotify/)


Note: You can download the readily available binary at https://github.com/sathwikmatsa/tcad/releases/download/0.1/tcad and skip the `Build` section.

## Build
```
> git clone https://github.com/sathwikmatsa/tcad.git
> cd tcad
> cargo build --release
```

## Configure
```
> cp .env_sample .env
```
substitue dummy values in `.env` with actuals

## Setup
After `cargo build`:
```
> mkdir -p ~/.bin/tcad
> cp target/release/tcad ~/.bin/tcad
> cp .env ~/.bin/tcad
```
Add a cron job
```
> crontab -e
```
append the following line* :
```
*/5 * * * * sudo -u $<user> DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/<uid>/bus /home/<user>/.bin/tcad/tcad /home/<user>/.bin/tcad/.env >> /home/<user>/.bin/tcad/tcad.log
```
to run tcad every `5 minutes`.

*Note: replace \<user\> and \<uid\> in the above line. (use `$whoami` to get user, `id -u $(whoami)` to get user id)
