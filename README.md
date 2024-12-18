# Wake on lan relay api

## Initial goals:

- [x]: Rust part, web server, wol and cli/config
- [x]: integration in dashboard or something
- [x]: frontend
- [x]: shutdown
- [x]: on off stage with ping
- [x]: docker ~compose~ stack deployment
- [x]: deploy on rpi
- [x]: auto deploy on push
- [ ]: connect vdi button
- [x]: launch apps/games with icons. (config entry with icon url and command to
  run)
- [ ]: config for running arbitrary command 'actions' on host/machine
- [ ]: machine agent reporting state + monitoring stats (cpu, memory, uptime/last online)
  - [ ]: can also be used as an alternative to ping/ssh
- [ ]: nix service agent
- [ ]: nixos package and nix ci
- [ ]: nixos module with a systemd service


## Resources
Might be interesting at some point
- https://motion.vueuse.org/api/use-motion
- https://sound.vueuse.org/

## Similar projects

- https://github.com/rix1337/WakeOnLAN-API/blob/main/wol_api/run.py
  super simple, similar to the project at the root
- https://github.com/seriousm4x/UpSnap
  Looks feature complete, beatifull ui, database, crud machine management, auto discovery and more
- https://github.com/Misterbabou/gptwol
  Simple, similar ui, not many features
