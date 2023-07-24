#!/bin/bash
set -eu

_die() {
    echo "err: $*"
    exit 1
}


COMPOSE="docker compose"
if ! $COMPOSE >/dev/null; then
    echo "could not call docker compose (hint: install docker compose plugin)"
    exit 1
fi
BCLI="$COMPOSE exec -T -u blits bitcoind bitcoin-cli -regtest"
DATA_DIR="./srv"

start() {
    $COMPOSE down -v
    rm -rf $DATA_DIR
    mkdir -p $DATA_DIR
    # see docker-compose.yml for the exposed ports
    EXPOSED_PORTS=(3000 50001)
    for port in "${EXPOSED_PORTS[@]}"; do
        if [ -n "$(ss -HOlnt "sport = :$port")" ];then
            _die "port $port is already bound, services can't be started"
        fi
    done
    $COMPOSE up -d

    # wait for bitcoind to be up
    until $COMPOSE logs bitcoind |grep 'Bound to'; do
        sleep 1
    done

    # prepare bitcoin funds
    $BCLI createwallet miner
    $BCLI -rpcwallet=miner -generate 103

    # wait for electrs to have completed startup
    until $COMPOSE logs electrs |grep 'finished full compaction'; do
        sleep 1
    done

    # wait for proxy to have completed startup
    until $COMPOSE logs proxy |grep 'App is running at http://localhost:3000'; do
        sleep 1
    done
}

stop() {
    $COMPOSE down -v
    rm -rf $DATA_DIR
}

[ -n "$1" ] || _die "command required"
case $1 in
    start|stop) "$1";;
    *) _die "unrecognized command";;
esac
