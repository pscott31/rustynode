CREATE DOMAIN HUGEINT AS NUMERIC(1000, 0);


create table accounts
(
    id        BYTEA PRIMARY KEY,
    party_id  BYTEA,
    asset_id  BYTEA  NOT NULL,
    market_id BYTEA,
    type      INT,
    tx_hash   BYTEA NOT NULL,
    vega_time TIMESTAMP WITH TIME ZONE NOT NULL,
    UNIQUE(party_id, asset_id, market_id, type)
);

CREATE TYPE transfer_type AS enum(
	'TRANSFER_TYPE_UNSPECIFIED',
	'TRANSFER_TYPE_LOSS',
	'TRANSFER_TYPE_WIN',
	'TRANSFER_TYPE_CLOSE',
	'TRANSFER_TYPE_MTM_LOSS',
	'TRANSFER_TYPE_MTM_WIN',
	'TRANSFER_TYPE_MARGIN_LOW',
	'TRANSFER_TYPE_MARGIN_HIGH',
	'TRANSFER_TYPE_MARGIN_CONFISCATED',
	'TRANSFER_TYPE_MAKER_FEE_PAY',
	'TRANSFER_TYPE_MAKER_FEE_RECEIVE',
	'TRANSFER_TYPE_INFRASTRUCTURE_FEE_PAY',
	'TRANSFER_TYPE_INFRASTRUCTURE_FEE_DISTRIBUTE',
	'TRANSFER_TYPE_LIQUIDITY_FEE_PAY',
	'TRANSFER_TYPE_LIQUIDITY_FEE_DISTRIBUTE',
	'TRANSFER_TYPE_BOND_LOW',
	'TRANSFER_TYPE_BOND_HIGH',
	'TRANSFER_TYPE_WITHDRAW_LOCK',
	'TRANSFER_TYPE_WITHDRAW',
	'TRANSFER_TYPE_DEPOSIT',
	'TRANSFER_TYPE_BOND_SLASHING',
	'TRANSFER_TYPE_STAKE_REWARD',
	'TRANSFER_TYPE_TRANSFER_FUNDS_SEND',
	'TRANSFER_TYPE_TRANSFER_FUNDS_DISTRIBUTE',
	'TRANSFER_TYPE_CLEAR_ACCOUNT',
	'TRANSFER_TYPE_CHECKPOINT_BALANCE_RESTORE'
);

create table ledger
(
    ledger_entry_time       TIMESTAMP WITH TIME ZONE NOT NULL,
    account_from_id bytea                      NOT NULL,
    account_to_id   bytea                      NOT NULL,
    --quantity        HUGEINT                  NOT NULL,
    quantity NUMERIC(1000, 0) not null,
    tx_hash         BYTEA                    NOT NULL,
    vega_time       TIMESTAMP WITH TIME ZONE NOT NULL,
    transfer_time   TIMESTAMP WITH TIME ZONE NOT NULL,
    type            transfer_type,
    PRIMARY KEY(ledger_entry_time)
);

create table balances
(
    account_id bytea                      NOT NULL,
    vega_time  TIMESTAMP WITH TIME ZONE NOT NULL,
    tx_hash    BYTEA NOT NULL,
    balance    HUGEINT           NOT NULL,
    PRIMARY KEY(vega_time, account_id)
);


CREATE TABLE orders (
    id                BYTEA                     NOT NULL,
    market_id         BYTEA                     NOT NULL,
    party_id          BYTEA                     NOT NULL, -- at some point add REFERENCES parties(id),
    side              SMALLINT                  NOT NULL,
    price             HUGEINT                    NOT NULL,
    size              BIGINT                    NOT NULL,
    remaining         BIGINT                    NOT NULL,
    time_in_force     SMALLINT                  NOT NULL,
    type              SMALLINT                  NOT NULL,
    status            SMALLINT                  NOT NULL,
    reference         TEXT,
    reason            SMALLINT,
    version           INT                       NOT NULL,
    batch_id          INT                       NOT NULL,
    pegged_offset     HUGEINT,
    pegged_reference  SMALLINT,
    lp_id             BYTEA,
    created_at        TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at        TIMESTAMP WITH TIME ZONE,
    expires_at        TIMESTAMP WITH TIME ZONE,
    tx_hash           BYTEA                    NOT NULL,
    vega_time         TIMESTAMP WITH TIME ZONE NOT NULL,
    seq_num           BIGINT NOT NULL,
    vega_time_to      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT 'infinity',
    PRIMARY KEY(vega_time, seq_num)
);
