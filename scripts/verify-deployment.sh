#!/bin/bash
set -e

source .stellar/deployed-tokens.env

echo "🔍 BondBridge Deployment Verification"
echo "======================================="
echo ""
echo "📋 Contract Addresses:"
echo "  BENJI:       $BENJI_TOKEN_ID"
echo "  USDC:        $USDC_TOKEN_ID"
echo "  Credit Line: $CREDIT_LINE_ID"
echo ""
echo "🌐 View on Stellar Expert:"
echo "  https://stellar.expert/explorer/testnet/contract/$BENJI_TOKEN_ID"
echo "  https://stellar.expert/explorer/testnet/contract/$USDC_TOKEN_ID"
echo "  https://stellar.expert/explorer/testnet/contract/$CREDIT_LINE_ID"
echo ""
echo "======================================="
echo ""

echo "💰 Token Balances:"
echo "-------------------"

ALICE_BENJI=$(stellar contract invoke \
  --id $BENJI_TOKEN_ID \
  --source alice \
  --network testnet \
  -- balance \
  --id $(stellar keys address alice) 2>&1 | tail -n1)

BOB_BENJI=$(stellar contract invoke \
  --id $BENJI_TOKEN_ID \
  --source bob \
  --network testnet \
  -- balance \
  --id $(stellar keys address bob) 2>&1 | tail -n1)

CREDIT_USDC=$(stellar contract invoke \
  --id $USDC_TOKEN_ID \
  --source deployer \
  --network testnet \
  -- balance \
  --id $CREDIT_LINE_ID 2>&1 | tail -n1)

DEPLOYER_USDC=$(stellar contract invoke \
  --id $USDC_TOKEN_ID \
  --source deployer \
  --network testnet \
  -- balance \
  --id $(stellar keys address deployer) 2>&1 | tail -n1)

echo "Alice BENJI:        $ALICE_BENJI (should be 100000000000)"
echo "Bob BENJI:          $BOB_BENJI (should be 50000000000)"
echo "Credit Line USDC:   $CREDIT_USDC (should be 500000000000)"
echo "Deployer USDC:      $DEPLOYER_USDC (should be 500000000000)"
echo ""

echo "✅ Verification complete!"