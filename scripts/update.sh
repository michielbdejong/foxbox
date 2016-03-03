#!/bin/bash
# Usage: ./update.sh my-link-box 192.168.0.42 52.36.71.23
echo "Got called: ./update.sh $1 $2 $3"

echo "Creating DNS records..."

node ./apiCall.js ns.useraddress.net 5300 /v1/dns/net/useraddress/$1 "{\"type\":\"A\",\"value\":\"$2\"}"
node ./apiCall.js ns.useraddress.net 5300 /v1/dns/net/useraddress/$1/a "{\"type\":\"A\",\"value\":\"$2\"}"
node ./apiCall.js ns.useraddress.net 5300 /v1/dns/net/useraddress/$1/b "{\"type\":\"A\",\"value\":\"$2\"}"
node ./apiCall.js ns.useraddress.net 5300 /v1/dns/net/useraddress/$1/remote "{\"type\":\"A\",\"value\":\"$2\"}"

echo "$1.useraddress.net a.$1.useraddress.net b.$1.useraddress.net remote.$1.useraddress.net" > ./domains.txt
echo "Getting SAN cert for: `cat domains.txt`"
./letsencrypt.sh --cron --challenge dns-01 --hook ./deploy-challenge.sh

echo "Setting remote. to use the tunnel"
node ./apiCall.js ns.useraddress.net 5300 /v1/dns/net/useraddress/$1/remote "{\"type\":\"A\",\"value\":\"$3\"}"
