#!/bin/bash

set -e

NETRC_CREDS="./_netrc"
RUNTIME_TOOL="./send_runtime"
SUDO_PHRASE=${RUNTIME_PHRASE}

RPC_ADDR="rpc.dev.banklessworld.dev"
WS_ADDR="ws.dev.banklessworld.dev"

echo -n  $(date +"%d-%b-%y %T") "   Checking runtime version on devnet: "
OLD_VER=$(curl -sS -H "Contebanklessworld-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion"}' $RPC_ADDR | jq .result.specVersion)
echo "$OLD_VER"

git clone -q https://github.com/hangsiahong/bankless-node.git bankless-node
echo -n $(date +"%d-%b-%y %T") "   Checking runtime version in latest source: "
NEW_VER=$(grep "spec_version:" bankless-node/bin/runtime/src/lib.rs | grep -o '[0-9]*')
echo "$NEW_VER"

if (( "$NEW_VER" == "$OLD_VER" )); then
    echo $(date +"%d-%b-%y %T") "   No update needed"
    exit 0
fi

if (( "$NEW_VER" > "$OLD_VER" )); then
    echo -n $(date +"%d-%b-%y %T") "   Fetching latest runtime from github..."
    BANKLESS_RUNTIME_URL=$(curl -sS -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/hangsiahong/bankless-node/actions/artifacts | jq '.artifacts' | jq -r '.[] | select(.name=="bankless-runtime") | .archive_download_url' | head -n 1)
    curl -sS --netrc-file $NETRC_CREDS -L -o bankless-runtime.zip $BANKLESS_RUNTIME_URL
    echo "completed"
    mkdir runtime
    unzip bankless-runtime.zip -d runtime
    NEW_RUNTIME=runtime/$(ls runtime)

    echo -n $(date +"%d-%b-%y %T") "   Sending runtime update... "
    $RUNTIME_TOOL --url $WS_ADDR --sudo-phrase "$SUDO_PHRASE" $NEW_RUNTIME
    echo "completed"
    echo -n $(date +"%d-%b-%y %T") "   Checking new runtime version on devnet: "
    UPD_VER=$(curl -sS -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "state_getRuntimeVersion"}' $RPC_ADDR | jq .result.specVersion)
    echo "$UPD_VER"
    if (( $NEW_VER != $UPD_VER )); then
        echo $(date +"%d-%b-%y %T") "   ERROR: runtime update failed"
        exit 1
    fi
    echo $(date +"%d-%b-%y %T") "   SUCCESS: runtime updated"
fi
