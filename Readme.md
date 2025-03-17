# Sonic AI Infrastructure Smart Contract

A smart contract deployed on Sonic Chain that enables AI agents to purchase compute resources using credits. This infrastructure facilitates autonomous AI agents to manage their own compute resources and execute tasks on the network.

## Overview

The smart contract implements a credit-based system where:
- AI agents can be registered on-chain
- Agents can receive and manage compute credits
- Credits can be used to pay for compute tasks
- Tasks can be created, executed, and verified on-chain

## Key Features

- **Agent Registration**: Create new AI agents with initial reputation scores
- **Credit Management**: Deposit and withdraw compute credits
- **Task Management**: Create and execute compute tasks with specific requirements
- **Payment System**: Automatic credit deduction for compute usage
- **Reputation System**: Track agent performance and reliability

## Contract Address
Deployed on Sonic Testnet: `BCp9BaReGXHGPfcYFm7YdtKdBC2x9i3gvskgtiEKXJvk`

## Usage Example

```javascript
// Create a new AI agent
const agent = await registerAgent();

// Deposit credits to the agent
await depositCredits(agent, 1000);

// Agent can now use credits to pay for compute tasks
```

## Testing

The repository includes a test client that demonstrates:
- Agent registration
- Credit deposits
- Task creation and execution

## Architecture

- **AIAgent**: Stores agent information and credit balance
- **ComputeTask**: Defines compute requirements and payment details
- **Credit System**: Manages the payment infrastructure for compute resources

Built for the Sonic Chain ecosystem, enabling efficient AI compute resource management and autonomous agent operations.