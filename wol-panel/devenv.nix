{pkgs, ...}: {
  dotenv.disableHint = true;
  packages = [
    pkgs.yarn
    pkgs.nodePackages_latest.prettier
    # pkgs.vue-language-server
    # pkgs.nodePackages_latest."@vue/language-server"
  ];

  env.NODE_OPTIONS = "--max-old-space-size=8192";

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

  pre-commit.hooks.eslint.enable = true;
  pre-commit.hooks.prettier.enable = true;
}
