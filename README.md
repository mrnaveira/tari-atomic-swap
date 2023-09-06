# tari-atomic-swap

This project is a proof of concept of a user-friendly atomic swap (HTLC) architecture between Ethereum and [Tari](https://github.com/tari-project/tari-dan).

It has three main components:
* A liquidity provider daemon (Rust application with a JSON RPC API)
* A swap web application (React)
* A matchmaking protocol between liquidity providers and web users, implemented using Tari templates

**This is NOT production ready yet, do not use it with real funds**.

## Getting started

### Minotari and Tari networks
This project needs both Minotari and Tari networks running:
* Minotari node and wallet (with funds for transaction fees)
* Tari validator node, indexer, signaling server, wallet daemon and wallet daemon web UI. Also, at least 2 accounts (for liquidity provider and swap user) with some funds.

### Tari templates
Inside the `networks/tari/templates` folder we can find all the Tari templates that need to be deployed to run this project:
* `atomic_swap`: for the HTLC implementation on the Tari side
* `lp_index`: matchmaking template where all liquidity providers link their overall info and web users scan
* `lp_position`: template for a particular liquidity provider's list of positions. The `lp_index` links each `lp_position` for each liquidity provider.

### Tari matchmaking component
The `lp_index` template must be initialized into a component, which both liquidity providers and swap users will use for discovery and matchmaking

The `lp_index` component can be initialized by calling the template function `new` using the wallet CLI (inside the `tari-dan` project):
```
$ cargo run --bin tari_dan_wallet_cli -- transactions submit --fee 5 --num-outputs 0  call-function -a template_<LP_POSITION_TEMPLATE_ADDRESS> <LP_INDEX_TEMPLATE_ADDRESS> new
```

The resulting component address will be needed later.


### Ethereum matchmaking component
For Ethereum HTLC smart contracts we are using [hashed-timelock-contract-ethereum](https://github.com/chatch/hashed-timelock-contract-ethereum).

You can deploy it to any Ethereum network. The smart contract repository uses Truffle, so the easiest way is to start [Ganache](https://trufflesuite.com/ganache/) on port `4447` and then run:
```
$ git clone https://github.com/chatch/hashed-timelock-contract-ethereum.git
$ cd hashed-timelock-contract-ethereum
$ npm install
$ npm run test
$ truffle migrate
```

There are a couple of possible problems when running it:
* [digital envelope routines::unsupported](https://stackoverflow.com/questions/69692842/error-message-error0308010cdigital-envelope-routinesunsupported)
* Truffle may not find the Ganache instance. Make sure that the `truffle-config.js` has the correct URL (you might need to change `localhost` to `127.0.0.1`).

We are going to need the deployed contract address of `HashedTimelock.sol` later.


### Liquidity provider daemon

Liquidity providers must run a daemon (`applications/liquidity_daemon`) that will:
* Syncs their positions with the matchmaking Tari component (`lp_index` template)
* Provides a JSON RPC interface with the operations for completing an atomic swap with the swap web application

First, we need to make a copy of the `config.json.example` file in the root of the LP application. Then edit the file to configure the provider's info and desired positions. Once the file is ready (we are going to assume it's named `config.json`), launch the liquidity provider daemon with:
```
$ cargo run -- -c config.json
```

### Swap web
Regular users can connect to a web page, link their Ethereum and Tari wallets and perform swaps using the liquidity that LP users provide.

To begin with, the `tari-connector` package must be installed in the global `npm` folder. To do it:
* Clone the `tari-connector` repository
* Run `npm install`
* Run `npm link`

The swap web needs some configuration (matchmaking component address, etc.). Inside the `applications/swap_web` project do:
* `$ cp .env.example .env`
* Edit the `.env` file with all the required information

Then, to run the web:
```
npm install
npm link tari-connector
npm run dev
```
