default:
    @just --list

export UID := `id -u`
export GID := `id -g`

run: run-docker
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-api && nix develop -c just watch -- -c ../dev/wol-config.yml" &
    alacritty -o 'font.size=10' -e sh -c "curl --retry-connrefused --connect-timeout 30 --retry 300 --retry-delay 1 'http://localhost:3030/api/machine/list' && firefox http://localhost:3000 http://localhost:3030/api/doc" &

run-docker:
    docker context use default
    alacritty -o 'font.size=10' -e sh -c "cd ./dev && docker compose down && docker compose up -d --build && docker compose logs -f front-tests" &

deploy:
    git push gitea
