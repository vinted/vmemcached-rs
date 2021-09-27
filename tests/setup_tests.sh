#!/bin/bash
set -e

BASEDIR=$(dirname "$0")
MEMCACHED_VERSION="1.6.9"
MEMCACHED_TARBALL="memcached-$MEMCACHED_VERSION.tar.gz"
MEMCACHED_DIR="$BASEDIR/memcached-$MEMCACHED_VERSION"
MEMCACHED="$MEMCACHED_DIR/memcached"

SSL_KEY=$BASEDIR/assets/localhost.key
SSL_CERT=$BASEDIR/assets/localhost.crt
SSL_ROOT_CERT=$BASEDIR/assets/RUST_MEMCACHE_TEST_CERT.crt

echo "Building memcached $MEMCACHED_VERSION with TLS support"
if [[ ! -d "$MEMCACHED_DIR" ]]; then
    curl "https://memcached.org/files/$MEMCACHED_TARBALL" -O
    tar xvf "$MEMCACHED_TARBALL" -C "$BASEDIR"
    rm "$MEMCACHED_TARBALL"
fi

if [[ ! -f "$MEMCACHED" ]]; then
    pushd "$MEMCACHED_DIR"
    ./configure --enable-tls
    make
    popd
fi

echo "Starting memcached server"
$MEMCACHED -V
$MEMCACHED -p 11211 -d
