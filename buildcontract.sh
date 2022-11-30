#!/bin/bash

#Build Flag

NETWORK=testnet
FUNCTION=$1
CATEGORY=blazarbit_protocol
PARAM_1=$3
PARAM_2=$4
PARAM_3=$5

ADDR_ACHILLES="juno15fg4zvl8xgj3txslr56ztnyspf3jc7n9j44vhz"
ADDR_MARBLE="osmo1hmtklnl8aque00rvewtd5pxve388zjkxwpg3wm"
ADDR_LOCAL="juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"
ADDR_BLOCK_TEST="juno190k40ya33qy3ssynwhlxcllgwlzk9j9gldc32p6y5c5scy4qhkdqxf3ene"
case $NETWORK in
  devnet)
    NODE="http://localhost:26657"
    DENOM=ujunox
    CHAIN_ID=testing
    LP_TOKEN_CODE_ID=1
    WALLET="--from local"
    ADDR_ADMIN=$ADDR_LOCAL
    ;;
  testnet)
    NODE="https://rpc-test.osmosis.zone:443"
    DENOM=uosmo
    CHAIN_ID=osmo-test-4
    LP_TOKEN_CODE_ID=123
    WALLET="--from vhdev"
    ADDR_ADMIN=$ADDR_MARBLE
    TOKEN_MARBLE="juno15s50e6k9s8mac9cmrg2uq85cgw7fxxfh24xhr0chems2rjxsfjjs8kmuje"
    ;;
  mainnet)
    NODE="https://rpc.osmosis.zone:443"
    DENOM=uosmo
    CHAIN_ID=osmosis-1
    LP_TOKEN_CODE_ID=1
    WALLET="--from vhdev"
    ADDR_ADMIN=$ADDR_MARBLE
    TOKEN_MARBLE="juno1g2g7ucurum66d42g8k5twk34yegdq8c82858gz0tq2fc75zy7khssgnhjl"
    TOKEN_BLOCK="juno1y9rf7ql6ffwkv02hsgd4yruz23pn4w97p75e2slsnkm0mnamhzysvqnxaq"
    ;;
esac

NODECHAIN=" --node $NODE --chain-id $CHAIN_ID"
TXFLAG=" $NODECHAIN --gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3"


RELEASE_DIR="artifacts/"

INFO_DIR="$NETWORK/"

FILE_CODE_CW721_BASE=$INFO_DIR"code_cw721_base.txt"
FILE_CODE_CW20_BASE=$INFO_DIR"code_cw20_base.txt"
FILE_CODE_MARBLE_COLLECTION=$INFO_DIR"code_marble_collection.txt"
FILE_CODE_MARBLE_MARKETPLACE=$INFO_DIR"code_marble_marketplace.txt"
FILE_CODE_NFTSALE=$INFO_DIR"code_nftsale.txt"
FILE_CODE_NFTSTAKING=$INFO_DIR"code_nftstaking.txt"

FILE_UPLOADHASH=$INFO_DIR"uploadtx.txt"
FILE_MARKETPLACE_CONTRACT_ADDR=$INFO_DIR"contract_marketplace.txt"
FILE_NFTSALE_ADDR=$INFO_DIR"contract_nftsale.txt"
FILE_NFTSTAKING_ADDR=$INFO_DIR"contract_nftstaking.txt"

AIRDROP_LIST_1="airdroplist/earlylp.json"
AIRDROP_LIST_2="airdroplist/final-daodao.json"
FILE_MERKLEROOT="merkleroot.txt"
###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################
#Environment Functions
CreateEnv() {
    sudo apt-get update && sudo apt upgrade -y
    sudo apt-get install make build-essential gcc git jq chrony -y
    wget https://golang.org/dl/go1.17.3.linux-amd64.tar.gz
    sudo tar -C /usr/local -xzf go1.17.3.linux-amd64.tar.gz
    rm -rf go1.17.3.linux-amd64.tar.gz

    export GOROOT=/usr/local/go
    export GOPATH=$HOME/go
    export GO111MODULE=on
    export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin
    
    rustup default stable
    rustup target add wasm32-unknown-unknown

    git clone https://github.com/CosmosContracts/juno
    cd juno
    git fetch
    git checkout v6.0.0
    make install
    cd ../
    rm -rf juno
}

RustBuild() {

    echo "================================================="
    echo "Rust Optimize Build Start"
    
    rm -rf release
    mkdir release
    
    #cd contracts
    
    #cd escrow
    RUSTFLAGS='-C link-arg=-s' cargo wasm
    cp target/wasm32-unknown-unknown/release/*.wasm release/

    # cd cw721-base
    # RUSTFLAGS='-C link-arg=-s' cargo wasm
    # cp target/wasm32-unknown-unknown/release/*.wasm ../../release/

    # cd ..
    #cd collection
    #RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
    #cp target/wasm32-unknown-unknown/release/*.wasm ../../release/

    #cd ..
    #cd marketplace
    #RUSTFLAGS='-C link-arg=-s' cargo wasm
    #cp target/wasm32-unknown-unknown/release/*.wasm ../../release/

    # cd ..
    # cd nftsale
    # RUSTFLAGS='-C link-arg=-s' cargo wasm
    # cp target/wasm32-unknown-unknown/release/*.wasm ./release/

    # cd nftstaking
    # RUSTFLAGS='-C link-arg=-s' cargo wasm
    # cp target/wasm32-unknown-unknown/release/*.wasm ../../release/

    #cd ../../
}

Upload() {
    echo "================================================="
    echo "Upload $CATEGORY.wasm to $NETWORK"
    UPLOADTX=$(osmosisd tx wasm store $RELEASE_DIR$CATEGORY".wasm" $WALLET $TXFLAG --output json  -b sync -y  | jq -r '.txhash')
    
    echo "Upload txHash:"$UPLOADTX
    
    echo "================================================="
    echo "GetCode"
	CODE_ID=""
    while [[ $CODE_ID == "" ]]
    do 
        sleep 3
        CODE_ID=$(osmosisd query tx $UPLOADTX $NODECHAIN --output json | jq -r '.logs[0].events[-1].attributes[1].value')
    done
    echo "Contract Code_id:"$CODE_ID

    #save to FILE_CODE_ID
    echo $CODE_ID > $INFO_DIR"code_"$CATEGORY".txt"
}
# {"name":"STKN","symbol":"STKN","decimals":6,"initial_balances":[],"mint":{"minter":"'$ADDR_ADMIN'"},"marketing":{"marketing":"'$ADDR_ADMIN'","logo":{"url":"https://i.ibb.co/RTRwxfs/prism.png"}}}


InstantiateSwapOsmo() {
    CODE_SWAPOSMO=4193
    TXHASH=$(osmosisd tx wasm instantiate $CODE_SWAPOSMO '{}' --label "MarblenautsSale$CODE_SWAPOSMO" --admin $ADDR_ADMIN $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    echo $TXHASH
    CONTRACT_ADDR=""
    while [[ $CONTRACT_ADDR == "" ]]
    do
        sleep 3
        CONTRACT_ADDR=$(junod query tx $TXHASH $NODECHAIN --output json | jq -r '.logs[0].events[0].attributes[0].value')
    done
    echo $CONTRACT_ADDR
    echo $CONTRACT_ADDR > $FILE_NFTSALE_ADDR
}

###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################

SwapOsmo2()
{
    osmosisd tx wasm execute osmo127f6lmrpvg779ljuv4td9ahhntne84t23newfnwd2wh6p3729pyqjs0llr '
    {
        "swap": {
            "pool_id": 1,
            "token_out_denom": "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2",
            "token_out_min_amount": "69000",
            "to": "osmo18g7ur8a9g5lt67ay955fks8q230kwd4gtwgz6c"
            }
        }
    ' --amount 1000uosmo $WALLET $TXFLAG -y    
}

TransferOsmo2()
{
    osmosisd tx wasm execute osmo1qpuph723gd3nyw646k7ft8lgr7pjl5wdfwfgaswrq9nd49pjfafq5tyh53 '
    {
        "transfer": {
            "address": "osmo15tl3jjhmf9tl2k6qkgfs3ky4lk9rfrkzr7ayv6"
            }
        }
    ' --amount 0uosmo $WALLET $TXFLAG -y    
}

AddCollection() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CODE_CW721_BASE=$(cat $FILE_CODE_CW721_BASE)

    junod tx wasm execute $CONTRACT_MARKETPLACE '
    {"add_collection": {
    "owner": "'$ADDR_ADMIN'",
    "max_tokens": 10000,
    "name": "Airdop11",
    "symbol": "BLOCK",
    "token_code_id": '$CODE_CW721_BASE',
    "maximum_royalty_fee": 100000,
    "royalties": [
      {
        "address": "'$ADDR_ADMIN'",
        "rate": 50000
      },
      {
        "address": "juno1jj9la354heml9f3f73gxkxhpyzzy6gfnsq582x",
        "rate": 10000
      }
    ],
    "uri": "QmfC1brvtFZFCRJGfQDCKTNLqo1wfSjdotWuG882X4cnM9"
  }}' $WALLET $TXFLAG -y

    # sleep 10
    
    # junod tx wasm execute $CONTRACT_MARKETPLACE '{"add_collection":{"owner": "'$ADDR_ADMIN'", "max_tokens": 1000000, "name": "Laocoön The Priest", "symbol": "MNFT","token_code_id": '$CODE_CW721_BASE',
    # "cw20_address": "'$TOKEN_MARBLE'",
    # "royalty": 0,
    # "uri": "https://marbledao.mypinata.cloud/ipfs/Qmf9jdbLfRbZQTfXu21u8UCj1Jp1y5GXHnBmMtmLnj1oUU"}}' $WALLET $TXFLAG -y
}

RemoveCollection() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    junod tx wasm execute $CONTRACT_MARKETPLACE '{"remove_collection":{"id": 8}}' $WALLET $TXFLAG -y
}

ListCollection() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    # junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"list_collections":{}}' $NODECHAIN
    junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":1}}' $NODECHAIN --output json
    TXHASH=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":1}}' $NODECHAIN --output json | jq -r '.data.collection_address')
    echo $TXHASH
}

Mint() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":1}}' $NODECHAIN --output json | jq -r '.data.collection_address')
    CONTRACT_CW721=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":1}}' $NODECHAIN --output json | jq -r '.data.cw721_address')

    junod tx wasm execute $CONTRACT_COLLECTION '{"mint": {"uri": "dddd"}}' $WALLET $TXFLAG -y
}
StartSale() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.collection_address')
    CONTRACT_CW721=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.cw721_address')

    # MSG='{"start_sale": {"sale_type": "Auction", "duration_type": {"Time":[300, 400]}, "initial_price":"100"}}'
    MSG='{"start_sale": {"sale_type": "Auction", "duration_type": {"Bid":5}, "initial_price":"10000", "reserve_price":"10000", "denom":{"native":"ujuno"}}}'
    #MSG='{"start_sale": {"sale_type": "Fixed", "duration_type": "Fixed", "initial_price":"100000", "reserve_price":"100000", "denom":{"native":"ujuno"}}}'
    ENCODEDMSG=$(echo $MSG | base64 -w 0)
    echo $ENCODEDMSG
    # sleep 3
# 
    junod tx wasm execute $CONTRACT_CW721 '{"send_nft": {"contract": "'$CONTRACT_COLLECTION'", "token_id":"439", "msg": "'$ENCODEDMSG'"}}' $WALLET $TXFLAG -y

}

CancelSale() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.collection_address')
    CONTRACT_CW721=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.-+.cw721_address')

    junod tx wasm execute $CONTRACT_COLLECTION '{"cancel_sale": {"token_id": 439}}' $WALLET $TXFLAG -y

}

StartStaking() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.collection_address')
    CONTRACT_CW721=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.cw721_address')
    echo $CONTRACT_CW721
    # MSG='{"start_sale": {"sale_type": "Auction", "duration_type": {"Time":[300, 400]}, "initial_price":"100"}}'
    MSG='{"stake": {}}'
    #MSG='{"start_sale": {"sale_type": "Fixed", "duration_type": "Fixed", "initial_price":"100000", "reserve_price":"100000", "denom":{"native":"ujuno"}}}'
    ENCODEDMSG=$(echo $MSG | base64 -w 0)
    echo $ENCODEDMSG
    # sleep 3
    junod tx wasm execute $CONTRACT_CW721 '{"send_nft": {"contract": "'$CONTRACT_COLLECTION'", "token_id":"439", "msg": "'$ENCODEDMSG'"}}' $WALLET $TXFLAG -y

}

PrintSale() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":1}}' $NODECHAIN --output json | jq -r '.data.collection_address')

    # junod query wasm contract-state smart $CONTRACT_COLLECTION '{"get_sales":{"start_after":0}}' $NODECHAIN
    junod query wasm contract-state smart $CONTRACT_COLLECTION '{"get_sales":{}}' $NODECHAIN
}


Propose() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":5}}' $NODECHAIN --output json | jq -r '.data.collection_address')

    # junod tx wasm execute $CONTRACT_COLLECTION '{"mint": {"uri": "dddd"}}' $WALLET $TXFLAG -y
    junod tx wasm execute $CONTRACT_COLLECTION '{"propose":{"token_id":439, "denom":"ujuno"}}' --amount 20000ujuno $WALLET $TXFLAG -y
}

Propose2() {
    CONTRACT_MARKETPLACE=$(cat $FILE_MARKETPLACE_CONTRACT_ADDR)
    CONTRACT_COLLECTION=$(junod query wasm contract-state smart $CONTRACT_MARKETPLACE '{"collection":{"id":3}}' $NODECHAIN --output json | jq -r '.data.collection_address')

    CONTRACT_BLOCK="juno1y9rf7ql6ffwkv02hsgd4yruz23pn4w97p75e2slsnkm0mnamhzysvqnxaq"
    MSG='{"propose":{"token_id":1}}'
    #MSG='{"start_sale": {"sale_type": "Fixed", "duration_type": "Fixed", "initial_price":"100000", "reserve_price":"100000", "denom":{"native":"ujuno"}}}'
    ENCODEDMSG=$(echo $MSG | base64 -w 0)

    # junod tx wasm execute $CONTRACT_COLLECTION '{"mint": {"uri": "dddd"}}' $WALLET $TXFLAG -y
    junod tx wasm execute $CONTRACT_BLOCK '{"send":{"contract":"'$CONTRACT_COLLECTION'", "amount":"11000000000", "msg":"'$ENCODEDMSG'"}}' $WALLET $TXFLAG -y
}


Test() {
    junod query wasm list-contract-by-code 365 $NODECHAIN --output json
}


#################################################################################
PrintWalletBalance() {
    echo "native balance"
    echo "========================================="
    junod query bank balances $ADDR_ADMIN $NODECHAIN
    echo "========================================="
}
ListCodes() {
    echo "========================================="
    junod query wasm list-contract-by-code 1143 $NODECHAIN
    echo "========================================="
}

Migrate() { 
    echo "================================================="
    echo "Migrate Contract"
    # juno1ker9z45q0zwryaupzruk5xg39su8tpv33u6ds6hlwpz3q5n28yvsp7e3uc
    # juno1ddyj0ycfmac3gn33hrpcayq7ver55l6hkgyuwwyfuxtfqx9nr9lsuujszj
    # juno16hjg4c5saxqqa3cwfx7aw9vzapqna7fn2xprttge888lw0zlw5us87nv8x
    # juno1ttk30ura2p79l7tu7n2ayltl8sfr2pzkmn52a7hnrrzf5w8tvewqlqkqq0
    # juno1pzv6qtmx8ud8hqm66g6vufxqshey9tcukwff6hx9m9c6nrhwl2zskxejga
    # juno1zhj6rz5fns0zryjdz4a2jlaan9ks7982kclucxn6qwd8u0n50lnsn2acld
    # juno1cxtq9w9sctnanzykphd2ac009k4fe2z43rv9lq5thde3tc2x3a0q764484

    # CONTRACT_ADDR=juno1hjmxtgc4dkmk3qvj2al0ct8c2z4f9envujs70yh7ngeu7rml804sgwkt84
    # echo $CONTRACT_ADDR
    
    
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1ker9z45q0zwryaupzruk5xg39su8tpv33u6ds6hlwpz3q5n28yvsp7e3uc 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1ddyj0ycfmac3gn33hrpcayq7ver55l6hkgyuwwyfuxtfqx9nr9lsuujszj 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno16hjg4c5saxqqa3cwfx7aw9vzapqna7fn2xprttge888lw0zlw5us87nv8x 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1ttk30ura2p79l7tu7n2ayltl8sfr2pzkmn52a7hnrrzf5w8tvewqlqkqq0 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1pzv6qtmx8ud8hqm66g6vufxqshey9tcukwff6hx9m9c6nrhwl2zskxejga 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1zhj6rz5fns0zryjdz4a2jlaan9ks7982kclucxn6qwd8u0n50lnsn2acld 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH
    # sleep 30
    # TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1cxtq9w9sctnanzykphd2ac009k4fe2z43rv9lq5thde3tc2x3a0q764484 1207 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    # echo $TXHASH

    TXHASH=$(printf "y\npassword\n" | junod tx wasm migrate juno1jxpkymvmvlp2lem650dx2tvqkpqd3js50j94hj8d8hjf2lu3wwyqvp39ja 1209 '{}' $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    echo $TXHASH
    
    
}
#################################### End of Function ###################################################
if [[ $FUNCTION == "" ]]; then
    # RustBuild
    # CATEGORY=cw20_base
    # printf "y\npassword\n" | Upload
    # CATEGORY=cw721_base
    # printf "y\npassword\n" | Upload
    # sleep 3
    # CATEGORY=marble_collection
    # printf "y\npassword\n" | Upload
    # sleep 3
    CATEGORY=marble_marketplace
    printf "y\npassword\n" | Upload

    # CATEGORY=nftsale
    # printf "y\npassword\n" | Upload

    # CATEGORY=nftstaking
    # printf "y\npassword\n" | Upload
    # sleep 3
    # InstantiateStaking
    # sleep 3
    # InstantiateMarble
    # printf "y\npassword\n" | InstantiateMarketplace
    # sleep 3
    # AddCollection
    # sleep 5
    # ListCollection
    # sleep 3
    # Mint
    # sleep 3
    
    # StartSale

else
    $FUNCTION $CATEGORY
fi
