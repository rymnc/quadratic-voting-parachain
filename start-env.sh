cargo build --release

cd ../polkadot && cargo build --release

cd ../polkadot && ./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-cfde.json

cd ../polkadot && ./target/release/polkadot --chain rococo-local-cfde.json --alice --tmp &

cd ../polkadot && ./target/release/polkadot --chain rococo-local-cfde.json --bob --tmp --port 30334 &

cd ../quadratic-voting-parachain && ./target/release/parachain-template-node export-genesis-state > genesis-state

# Export genesis wasm
cd ../quadratic-voting-parachain && ./target/release/parachain-template-node export-genesis-wasm > genesis-wasm

# Collator1
cd ../quadratic-voting-parachain && ./target/release/parachain-template-node --collator --alice --force-authoring --tmp --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30335 &

# Collator2
cd ../quadratic-voting-parachain && ./target/release/parachain-template-node --collator --bob --force-authoring --tmp --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30336 &

# Parachain Full Node 1
cd ../quadratic-voting-parachain && ./target/release/parachain-template-node --tmp --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-cfde.json --port 30337 &