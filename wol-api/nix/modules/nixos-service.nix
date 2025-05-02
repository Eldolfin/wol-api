{
  self,
  inputs,
  ...
}: let
  nixosModule = {
    config,
    lib,
    pkgs,
    ...
  }:
    with lib; let
      cfg = config.eldolfin.services.wol-agent;
    in {
      options.eldolfin.services.wol-agent = {
        enable = mkEnableOption "Enables the wol-agent service";

        domain = mkOption {
          type = types.str;
          example = "192.168.1.1:3001";
          description = "The domain name for the wol-backend";
        };

        machine-name = mkOption {
          type = types.str;
          example = "tour";
          description = "The machine name identify as to the backend";
        };
      };

      config =
        mkIf cfg.enable
        (let
          configFile = pkgs.writeTextFile {
            name = "agent-config.yml";
            text = ''
              start_vdi_cmd: "echo removed vdi!"
              machine_name: "${cfg.machine-name}"
              domain: "${cfg.domain}"
            '';
          };
        in {
          systemd.services."eldolfin.wol-agent" = {
            wantedBy = ["multi-user.target"];
            environment = {
              RUST_LOG = "debug";
              # hardcode... 🙄
              XDG_DATA_DIRS = "/home/oscar/.nix-profile/share:/nix/profile/share:/home/oscar/.local/state/nix/profile/share:/etc/profiles/per-user/oscar/share:/nix/var/nix/profiles/default/share:/run/current-system/sw/share:/home/oscar/.local/share/applications";
              DISPLAY = ":0";
            };

            serviceConfig = let
              pkg = self.packages.${pkgs.system}.default;
            in {
              Restart = "always";
              RestartSec = 2;
              ExecStart = "!${pkg}/bin/agent ${configFile}";
              User = "oscar";
              # DynamicUser = "yes";
              RuntimeDirectory = "eldolfin.wol-agent";
              RuntimeDirectoryMode = "0755";
              StateDirectory = "eldolfin.wol-agent";
              StateDirectoryMode = "0700";
              CacheDirectory = "eldolfin.wol-agent";
              CacheDirectoryMode = "0750";
            };
          };
        });
    };
in {
  flake.nixosModules.default = nixosModule;
}
