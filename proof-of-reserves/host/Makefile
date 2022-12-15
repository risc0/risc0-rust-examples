get-tx-out-merkle-proof:
	curl $(rpc_url)/ \
    -X POST \
    -H "Content-Type: application/json" \
    --data '{"method": "gettxoutproof", "params": [["$(tx_hash)"]]}'