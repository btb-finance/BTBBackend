BTB Token Sale Smart Contract

Overview

This project is a Solana smart contract for managing a token sale where users can buy BTB tokens using various accepted payment tokens. Only Admins can initialize the sale, manage active payment tokens, and monitor transactions. Users can purchase tokens based on real-time prices.

Prerequisites

Rust v1.79.0+
Solana CLI v1.18.0
Anchor v0.29.0 or v0.30.0
Instructions

Build the Project
To build the smart contract:

anchor build


Deploy the Program
To deploy to a Solana cluster:

anchor deploy

Test the Program
To run tests:

anchor test



Functions

Initialize Sale: Admin sets up the token sale with pricing, payment options, and team wallet
Update Sale: Admin updates the sale’s active status.
Update Token Status: Admin manages the activity status of each payment token.
Buy Tokens: Users can purchase BTB tokens using available payment tokens.
Directory Structure








