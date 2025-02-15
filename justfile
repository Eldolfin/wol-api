default:
    @just --list

export UID := `id -u`
export GID := `id -g`

run: run-docker
    $TERM_PROGRAM --font-size=12 -e "cd ./wol-api && nix develop -c just watch-backend" &>/dev/null & disown
    $TERM_PROGRAM --font-size=12 -e "cd ./wol-api && nix develop -c just watch-agent" &>/dev/null & disown

    $TERM_PROGRAM --font-size=12 -e "curl --retry-connrefused --connect-timeout 30 --retry 300 --retry-delay 1 'http://localhost:3030/api/machine/list' && chromium http://localhost:3000 http://localhost:3030/api/doc" &>/dev/null & disown

run-docker:
    docker context use default
    $TERM_PROGRAM --font-size=12 -e "just start-docker-here" &>/dev/null & disown

start-docker-here:
    cd ./dev && docker compose down && docker compose --env-file .env up -d --build

deploy:
    git push gitea
