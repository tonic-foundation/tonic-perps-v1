import {
  ONE_USD, TTA_DEC, NEAR_ID, near, dollars, updateIndexPrice, test, denom,
  mintLp, mintLpFtTransfer, ftBalanceOf, TTB_DEC, LP_DEC, DOLLAR_DEC, swap, setFeeParameters
}
  from './utils.ava';

test('swap', async (t) => {
  const { root, tta, ttb, contract } = t.context.accounts;

  const ONE_USD = BigInt('1000000');
  const priceA = ONE_USD;
  const priceB = BigInt(2) * ONE_USD;

  await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
  await updateIndexPrice(root, contract, ttb.accountId, priceB.toString());

  const amtB = BigInt(50000);

  // deposit B to fill the pool
  await mintLpFtTransfer(root, contract, ttb, amtB.toString());

  const valueB = amtB * BigInt(priceB) / denom(TTB_DEC);
  const expectedLp0 = denom(LP_DEC) * valueB / denom(DOLLAR_DEC);

  t.is(
    BigInt(await ftBalanceOf(contract, root)),
    expectedLp0,
  );

  const amtBeforeA = BigInt(await ftBalanceOf(tta, root));
  const amtBeforeB = BigInt(await ftBalanceOf(ttb, root));

  await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
  await updateIndexPrice(root, contract, ttb.accountId, priceB.toString());

  // swap A for B
  const amtIn = BigInt(10);
  await swap(root, contract, tta, amtIn.toString(), ttb.accountId);

  t.is(BigInt(await ftBalanceOf(tta, root)), amtBeforeA - amtIn);

  const expectedAmtOut = amtIn * denom(TTB_DEC) * priceA / priceB / denom(TTA_DEC);
  t.is(BigInt(await ftBalanceOf(ttb, root)), amtBeforeB + expectedAmtOut);
});

test('swap fees', async (t) => {
  const { root, tta, ttb, contract } = t.context.accounts;

  const nearPrice = dollars(5);

  await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
  await updateIndexPrice(root, contract, ttb.accountId, dollars(2));
  await updateIndexPrice(root, contract, NEAR_ID, nearPrice);

  let amountB = BigInt(500) * denom(TTB_DEC);

  // deposit B to fill the pool
  await mintLpFtTransfer(root, contract, ttb, amountB.toString());

  // Add NEAR liquidity
  await mintLp(root, contract, near(300));

  await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
  await updateIndexPrice(root, contract, ttb.accountId, dollars(2));
  await updateIndexPrice(root, contract, NEAR_ID, nearPrice);

  let amount1 = BigInt(100) * denom(TTA_DEC); // $100

  await setFeeParameters(root, contract, 10);

  // swap A for B
  await swap(root, contract, tta, amount1.toString(), ttb.accountId);

  await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
  await updateIndexPrice(root, contract, ttb.accountId, dollars(2));
  await updateIndexPrice(root, contract, NEAR_ID, nearPrice);

  let amount2 = BigInt(500) * denom(TTA_DEC); // $500

  // swap A for NEAR
  await swap(root, contract, tta, amount2.toString(), NEAR_ID);

  await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
  await updateIndexPrice(root, contract, ttb.accountId, dollars(2));
  await updateIndexPrice(root, contract, NEAR_ID, nearPrice);

  const balanceBefore = await root.availableBalance();
  const ttbBalanceBefore = await ftBalanceOf(ttb, root);
  const fees: { asset_id: string, token_amount: string; usd: string}[] = await contract.view('get_assets_fee');
  const nearFee = fees.find(fee => fee.asset_id == NEAR_ID);
  const ttbFee = fees.find(fee => fee.asset_id == ttb.accountId);

  let transferAmount: string = await root.call(contract, 'withdraw_fees', {});

  const balanceAfter = await root.availableBalance();
  const ttbBalanceAfter = await ftBalanceOf(ttb, root);

  t.is(BigInt(nearFee?.usd!) + BigInt(ttbFee?.usd!), BigInt(transferAmount));
  // Small loss because of sandbox issues
  t.assert(
    BigInt(nearFee?.token_amount!) - BigInt(balanceAfter.sub(balanceBefore).toString())
    <= BigInt('2000000000000000000000') // 0.002 NEAR
  );
  t.is(BigInt(ttbBalanceAfter) - BigInt(ttbBalanceBefore), BigInt(ttbFee?.token_amount!));

  const fees2: { usd: string; }[] = await contract.view('get_assets_fee');
  let totalUsd = BigInt(0);
  fees2.forEach((asset) => totalUsd += BigInt(asset.usd));

  t.is(totalUsd, BigInt(0));
});