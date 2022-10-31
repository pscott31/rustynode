proto:
	protoc --rust_out src/protos \
	 src/protos/sources/vega/events/v1/events.proto\
	 src/protos/sources/vega/chain_events.proto \
	 src/protos/sources/vega/vega.proto \
	 src/protos/sources/vega/assets.proto \
	 src/protos/sources/vega/markets.proto \
	 src/protos/sources/vega/oracles/v1/data.proto \
	 src/protos/sources/vega/oracles/v1/spec.proto \
	 src/protos/sources/vega/governance.proto \
	 src/protos/sources/vega/commands/v1/commands.proto \
	 src/protos/sources/vega/commands/v1/oracles.proto \
	 src/protos/sources/vega/commands/v1/signature.proto \
	 src/protos/sources/vega/commands/v1/validator_commands.proto \
	  -I src/protos/sources