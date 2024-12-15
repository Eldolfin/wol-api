# Wake on lan relay api

## TODO:

- [x]: Rust part, web server, wol and cli/config
- [ ]: nixos package and nix ci
- [ ]: nixos module with a systemd service
- [x]: integration in dashboard or something
- [x]: frontend
- [x]: shutdown
- [ ]: on off stage with ping
- [ ]: connect vdi button
- [x]: docker ~compose~ stack deployment
- [x]: deploy on rpi
- [x]: auto deploy on push
- [ ]: machine agent reporting state + monitoring stats
- [ ]: nix service agent
- [ ]: launch apps/games with icons. (config entry with icon url and command to
  run)
  - [ ]: run command once health check succeed 

rewrite of this in rust 🦀
https://github.com/rix1337/WakeOnLAN-API/blob/main/wol_api/run.py

## Resources
Might be interesting at some point
- https://motion.vueuse.org/api/use-motion
- https://sound.vueuse.org/

## Similar projects

- https://github.com/seriousm4x/UpSnap
  Looks feature complete, beatifull ui, database, crud machine management, auto discovery and more
- https://github.com/Misterbabou/gptwol
  Simple, similar ui, not many features
