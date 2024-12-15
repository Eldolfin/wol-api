default:
    @just --list

run:
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-panel && npm run dev" &
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-api && nix develop -c just watch -- -c examples/config.yml -n" &

deploy:
    git push gitea
