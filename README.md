[![LOGO][bankless-logo]][bankless-homepage]

[![Unit tests][unit-tests-badge]][unit-tests]
[![E2E Tests][e2e-tests-badge]][e2e-tests]
[![Apache 2.0 Licensed][license-image]][license-link]


This repository contains the Rust implementation of [Bankless World][bankless-homepage] blockchain node based on the [Substrate][substrate-homepage] framework.

Bankless World is an open-source layer 1 blockchain focused on privacy, scalability and energy efficiency. It is based on a unique, peer-reviewed consensus algorithm, BanklessBFT (as described in our [paper][bankless-bft-paper] and implemented [here][bankless-bft-link]).

Bankless node is based on a Substrate node where the default finality gadget (GRANDPA) has been replaced with BanklessBFT. Block authoring is realized with Substrate's Aura. The chain is run in periodic sesssions (900 blocks each) utilizing the Session pallet. The authorities in each session serve for both Aura and BanklessBFT authorities, and on top of that are responsible for running the Aggregator protocol producing multisignatures of finalized blocks.


### Installation

1. Install the Substrate framework as described [here][substrate-installation], using the `nightly-2021-07-21` version of the rust toolchain (see the instruction at the bottom of the page)
2. Clone this repository to your machine
3. Run `cargo build --release -p bankless-node`

### Running

To experiment with Bankless Node you can locally run a small blockchain network using the `run_nodes.sh` script from the root of this repository.  Please consult the script or the output of `run_nodes.sh -help` for additional parameters (like the number of nodes etc.). The script starts multiple instances of Bankless Node on your local machine, so please adjust the number of nodes carefully with performance of your system in mind. By default 4 nodes are started. 

You can interact with your locally running nodes using RPC (use port 9933 for node0, 9934 for node1 and so on). A more convenient alternative is to attach to it with a polkadot.js wallet app. We recommend using our fork of that app which can be found [here][bankless-polkadot-link].

### Contributing

If you would like to contribute, please fork the repository, introduce your changes and submit a pull request. All pull requests are warmly welcome.

### License

The code in this repository is licensed under the terms of the the Apache License 2.0.


[bankless-homepage]: https://banklessworld.org
[bankless-logo]: https://banklessworld.org/wp-content/uploads/A0_logotype_dark-1.jpg
[bankless-bft-link]: https://github.com/Cardinal-Cryptography/BanklessBFT
[bankless-bft-paper]: https://arxiv.org/abs/1908.05156
[bankless-polkadot-link]: https://github.com/Cardinal-Cryptography/apps
[substrate-homepage]: https://substrate.io
[substrate-installation]: https://docs.substrate.io/v3/getting-started/installation
[rust-installation]: https://www.rust-lang.org/tools/install

[unit-tests]: https://github.com/Cardinal-Cryptography/bankless-node/actions/workflows/unit_tests.yml
[unit-tests-badge]: https://github.com/Cardinal-Cryptography/bankless-node/actions/workflows/unit_tests.yml/badge.svg
[e2e-tests]: https://github.com/Cardinal-Cryptography/bankless-node/actions/workflows/e2e-tests-main-devnet.yml
[e2e-tests-badge]: https://github.com/Cardinal-Cryptography/bankless-node/actions/workflows/e2e-tests-main-devnet.yml/badge.svg
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/Cardinal-Cryptography/bankless-node/blob/main/LICENSE
