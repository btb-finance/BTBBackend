# BTBBackendV1



## Overview

BTBBackendV1 is a sophisticated backend service designed for managing decentralized finance (DeFi) operations on the Raydium platform. This service provides automated liquidity management, including adding and removing liquidity, claiming rewards, rebalancing, and reinvesting funds. It empowers users to customize how their earned fees are managed, offering options to reinvest all or part of their fees back into liquidity pools or to claim their fees in any token of their choice.

## Features

- **🌊 Add Liquidity**: Seamlessly add liquidity to Raydium pools.
- **🔄 Remove Liquidity**: Effortlessly remove liquidity from pools as needed.
- **🏆 Claim Rewards**: Automatically claim rewards and update positions.
- **⚖️ Rebalance Liquidity**: Automate the rebalancing of liquidity positions based on predefined conditions.
- **💰 Reinvest Rewards**: Reinvest claimed rewards back into liquidity pools or convert them to preferred tokens.
- **🎛️ Custom Fee Management**:
  - **Reinvest Fees**: Choose to reinvest all or a specific percentage of earned fees back into the pool.
  - **Claim Fees in Preferred Tokens**: Opt to claim fees in any preferred token, rather than the pool's native token.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js and npm](https://nodejs.org/en/download/)
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/btb-finance/BTBBackendV1.git
   cd BTBBackendV1
   ```

2. Install dependencies:
   ```bash
   anchor build
   ```

### Quick Start

1. Deploy the program:
   ```bash
   anchor deploy
   ```

2. Run tests:
   ```bash
   anchor test
   ```

## Building and Deployment

### Building the Project

To build the project, run:

```bash
anchor build
```

### Deploying to Solana Devnet

To deploy the program to the Solana Devnet:

```bash
anchor deploy
```

### Deploying to Solana Mainnet

To deploy to the Solana Mainnet:

1. Set the Solana cluster to Mainnet:
   ```bash
   solana config set --url https://api.mainnet-beta.solana.com
   ```

2. Deploy the program:
   ```bash
   anchor deploy
   ```

## Testing

Run the included tests to ensure everything is working correctly:

```bash
anchor test
```

This will compile and run the tests located in the `tests/` directory.

## Contributing

We welcome contributions to BTBBackendV1! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

If you encounter any issues or have questions, please file an issue on our [GitHub issue tracker](https://github.com/btb-finance/BTBBackendV1/issues).

---

Made with ❤️ by the BTB Finance Team
