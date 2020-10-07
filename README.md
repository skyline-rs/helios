# Helios

A skyline-based plugin for automatically keeping your mods up-to-date. If an update is available, it prompts you to install it on launch.


### Usage

To create a mod to be updated via Helios, simply create a simple config:

```toml
name = "helios_test"
version = "1.0.0"
server_ip = "999.999.999.999"
```

Then name it `[name here].toml` and throw it in `sd:/helios/[title id of game]/`. Helios will automatically keep your config file version up to date.

Config fields:

* `name` (required) - the name of the plugin as present on the skyline-update server
* `version` (required) - the version of the plugin being included on the SD card in the form of `MAJOR.MINOR.PATCH`.
* `server_ip` (required) - the IP address of the server to install the update from. The server should be running skyline-update's update-server.

It is recommended that mod creator's should include the config file with the download itself. This will allow anyone who installs the mod and has helios installed to update the mod without even thinking about it.
