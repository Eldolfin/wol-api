{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  dotenv.disableHint = true;
  packages = [
    pkgs.yarn
    # pkgs.vue-language-server
    # pkgs.nodePackages_latest."@vue/language-server"
  ];

  languages.typescript.enable = true;
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
