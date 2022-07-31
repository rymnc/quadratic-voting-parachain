tmux new-session \; \
  send-keys '../polkadot/target/release/polkadot --chain ../polkadot/rococo-local-cfde.json --alice --tmp' C-m \; \
  split-window -v \; \
  send-keys '../polkadot/target/release/polkadot --chain ../polkadot/rococo-local-cfde.json --bob --tmp --port 30334' C-m \; \
  split-window -v\; \
  send-keys './target/release/parachain-template-node export-genesis-state  > genesis-state' C-m \; \
  send-keys './target/release/parachain-template-node export-genesis-wasm  > genesis-wasm' C-m \; \
  send-keys './target/release/parachain-template-node --collator --alice --force-authoring --tmp --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30335' C-m \; \
  split-window -v \; \
  send-keys './target/release/parachain-template-node --collator --bob --force-authoring --tmp --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30336' C-m \; \
  select-pane -t 0\; \
  split-window -v\; \
  send-keys './target/release/parachain-template-node --tmp --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337' C-m \; \