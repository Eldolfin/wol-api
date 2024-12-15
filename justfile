default:
    @just --list

run:
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-panel && npm run dev" &
    alacritty -o 'font.size=10' -e sh -c "cd ./wol-api && nix develop -c just watch -- -c ../test-machine/wol-config.yml" &
    docker context use default
    cd ./test-machine && docker compose up -d --wait
    curl \
        --retry-connrefused \
        --connect-timeout 30 \
        --retry 300 \
        --retry-delay 1 \
        "http://localhost:3030/api/machine/list"
    firefox http://localhost:3000 http://localhost:3030/api/rapidoc

deploy:
    git push gitea
