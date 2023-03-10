import {
    ONE_USD, TTA_DEC, NEAR_ID, near, dollars, updateIndexPrice, test, denom, ONE_NEAR,
    setFeeParameters, increasePosition, mintLp, mintLpFtTransfer, liquidatePosition, ftBalanceOf, NEAR_DEC
} from './utils.ava';

test('liquidation: short position', async (t) => {

    const { root, tta, contract, alice } = t.context.accounts;

    const priceA = ONE_USD;
    const priceNEAR = dollars(5);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, NEAR_ID, priceNEAR);

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Add stable liquidity
    await mintLpFtTransfer(root, contract, tta, '100000000000'); // $1000

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, NEAR_ID, priceNEAR);

    // Create short position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), false, near(30));

    const position: { collateral: string; size: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(10));

    const status: { insolvent: boolean; max_leverage_exceeded: boolean; } = await contract.view('get_liquidation_status', {
        position_id: positionId
    });
    const positionBeforeLiquidation: { value: string; collateral: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(status.insolvent, true);
    t.is(status.max_leverage_exceeded, false);
    t.is(positionBeforeLiquidation.value, '0');

    await root.call(contract, 'set_private_liquidation_only', {
        private_liquidation_only: false,
    });

    const nearAssetBefore: { pool_amount: string; } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetBefore: { pool_amount: string; } = await contract.view('get_asset_info', {
        asset: tta
    });
    const aliceBalanceBefore: string = await ftBalanceOf(tta, alice);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(10));

    await liquidatePosition(alice, contract, positionId);

    const nearAssetAfter: { pool_amount: string; } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetAfter: { pool_amount: string; average_price: string } = await contract.view('get_asset_info', {
        asset: tta
    });
    const positionAfterLiquidation = await contract.view('get_position', {
        position_id: positionId
    });
    const aliceBalanceAfter: string = await ftBalanceOf(tta, alice);

    const collateralAmount = BigInt(positionBeforeLiquidation.collateral) * BigInt(denom(TTA_DEC)) / BigInt(ttaAssetAfter.average_price);
    // Default liquidation reward is 10%
    const rewardAmount = BigInt(collateralAmount) / BigInt(10);

    t.is(positionAfterLiquidation, null);
    t.is(nearAssetBefore.pool_amount, nearAssetAfter.pool_amount);
    // Remove liquidator's reward from a pool, add short position collateral
    t.is(
        BigInt(ttaAssetBefore.pool_amount) - rewardAmount + collateralAmount,
        BigInt(ttaAssetAfter.pool_amount)
    );
    t.is(
        BigInt(aliceBalanceBefore) + rewardAmount,
        BigInt(aliceBalanceAfter)
    );
});

test('liquidation: long position', async (t) => {
    const { root, alice, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(10));

    const position: { collateral: string; size: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(50));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(1));

    const status: { insolvent: boolean; max_leverage_exceeded: boolean; } = await contract.view('get_liquidation_status', {
        position_id: positionId
    });
    t.is(status.insolvent, true);
    t.is(status.max_leverage_exceeded, false);

    await root.call(contract, 'set_private_liquidation_only', {
        private_liquidation_only: false,
    });

    const nearAssetBefore: { pool_amount: string; } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const balanceBefore = await alice.availableBalance();
    const positionBeforeLiquidation: { collateral: string; } = await contract.view('get_position', {
        position_id: positionId
    });

    await updateIndexPrice(root, contract, NEAR_ID, dollars(1));

    await liquidatePosition(alice, contract, positionId);

    const nearAssetAfter: { pool_amount: string; average_price: string; } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const positionAfterLiquidation = await contract.view('get_position', {
        position_id: positionId
    });
    const balanceAfter = await alice.availableBalance();

    const collateralAmount = BigInt(positionBeforeLiquidation.collateral) * BigInt(denom(NEAR_DEC)) / BigInt(nearAssetAfter.average_price);
    // Default liquidation reward is 10%
    const rewardAmount = BigInt(collateralAmount) / BigInt(10);

    t.is(positionAfterLiquidation, null);
    // Remove liquidator's reward from a pool, long position collateral is already 
    // a part of a pool.
    t.is(
        BigInt(nearAssetBefore.pool_amount) - rewardAmount,
        BigInt(nearAssetAfter.pool_amount)
    );
    t.assert(
        rewardAmount - BigInt(balanceAfter.sub(balanceBefore).toString())
        <= BigInt('900000000000000000000') // 0.0009 NEAR
    );
});

test('liquidation: max leverage error', async (t) => {
    const { root, alice, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add NEAR liquidity
    await mintLp(root, contract, near(1000));

    const sizeDelta = dollars(2000);

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, sizeDelta, true, near(60));

    await setFeeParameters(root, contract, 10);

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    const position: { collateral: string; size: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(300));
    t.is(position.size, dollars(2000));

    let price = BigInt(dollars(45)) / BigInt(10);

    await updateIndexPrice(root, contract, NEAR_ID, price.toString());

    const status: { insolvent: boolean; max_leverage_exceeded: boolean; } = await contract.view('get_liquidation_status', {
        position_id: positionId
    });
    t.is(status.insolvent, false);
    t.is(status.max_leverage_exceeded, true);

    await root.call(contract, 'set_private_liquidation_only', {
        private_liquidation_only: false,
    });

    const nearAssetBefore: { pool_amount: string; accumulated_fees: string } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const positionBeforeLiquidation: { size: string; } = await contract.view('get_position', {
        position_id: positionId
    });

    await updateIndexPrice(root, contract, NEAR_ID, price.toString());

    await liquidatePosition(alice, contract, positionId);

    const nearAssetAfter: { pool_amount: string; accumulated_fees: string; } = await contract.view('get_asset_info', {
        asset: NEAR_ID
    });
    const positionAfterLiquidation: { size: string; } = await contract.view('get_position', {
        position_id: positionId
    });

    t.assert(BigInt(positionBeforeLiquidation.size) > BigInt(positionAfterLiquidation.size));
    t.assert(BigInt(nearAssetBefore.pool_amount) >= BigInt(nearAssetAfter.pool_amount));
    t.assert(BigInt(nearAssetBefore.accumulated_fees) <= BigInt(nearAssetAfter.accumulated_fees));
});
