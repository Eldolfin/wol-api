{self, ...}: let
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
          default = "";
          example = "192.168.1.1:3001";
          description = "The domain name for the wol-backend";
        };
      };

      config = mkIf cfg.enable {
        systemd.services."eldolfin.wol-agent" = {
          wantedBy = ["multi-user.target"];

          serviceConfig = let
            pkg = self.packages.${pkgs.system}.default;
          in {
            Restart = "on-failure";
            ExecStart = "${pkg}/bin/agent ${cfg.domain}";
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
