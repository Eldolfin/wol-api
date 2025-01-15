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

        sanzupkg = mkOption {
          type = types.str;
          example = "sanzu.default";
          description = "Package to use for vdi";
          default = inputs.sanzu.default;
        };
      };

      config = mkIf cfg.enable {
        configFile = writeTextFile {
          name = "agent-config.yml";
          text = ''
            start_vdi_cmd = "${config.sanzupkg}/bin/sanzu_server -f ${config.sanzupkg.default-config} -e h264_nvenc"
            machine_name = "${cfg.machine-name}"
            domain = "${cfg.domain}"
          '';
        };
        systemd.services."eldolfin.wol-agent" = {
          wantedBy = ["multi-user.target"];

          serviceConfig = let
            pkg = self.packages.${pkgs.system}.default;
          in {
            Restart = "on-failure";
            ExecStart = "${pkg}/bin/agent --config ${configFile}";
            DynamicUser = "yes";
            RuntimeDirectory = "eldolfin.wol-agent";
            RuntimeDirectoryMode = "0755";
            StateDirectory = "eldolfin.wol-agent";
            StateDirectoryMode = "0700";
            CacheDirectory = "eldolfin.wol-agent";
            CacheDirectoryMode = "0750";
            Environment = "RUST_LOG=debug";
          };
        };
      };
    };
in {
  flake.nixosModules.default = nixosModule;
}
