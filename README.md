# tari-atomic-swap

This project is a proof of concept of an atomic swap (HTLC) between Ethereum and [Tari](https://github.com/tari-project/tari-dan).

**This is NOT production ready yet, do not use it with real funds**.

## Getting started

### Ethereum side

For HTLC smart contracts we are using [hashed-timelock-contract-ethereum](https://github.com/chatch/hashed-timelock-contract-ethereum).

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

### Tari side

Prerequisites:
* You need to have access to a wallet daemon connected to a Tari network. The wallet must have two accounts (Alice and Bob) with enough funds for the swap.
* Deploy the atomic swap template under `./networks/tari/templates/atomic_swap`.

### Running the swap POC

With all the previous steps in place:
* Move to the swap application: `$ cd applications/poc`
* Create a `dotenv` file with the configuration: `$ cp .env.example .env`
* Set the values of all the required variables in the `.env` file.
* Run `cargo run`
