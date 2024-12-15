default:
    @just --list

run:
    docker context use default
    (cd ./test-machine && docker compose up -d) &
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-panel && npm run dev" &
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-api && nix develop -c just watch -- -c ../test-machine/wol-config.yml" &

deploy:
    git push gitea
