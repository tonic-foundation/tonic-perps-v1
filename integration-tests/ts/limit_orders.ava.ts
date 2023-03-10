import {
    addDecreaseLimitOrder, addIncreaseLimitOrder, NEAR_ID, near, dollars, updateIndexPrice, test,
    increasePosition, mintLp, mintLpFtTransfer
}
    from './utils.ava';

test('limit orders: add and remove', async (t) => {

    const { root, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(30));

    const position: { collateral: string; size: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    const limitOrderId = await addDecreaseLimitOrder(root, contract, dollars(4), dollars(400), NEAR_ID, NEAR_ID, true, dollars(150), '0');

    const userOrders: any[] = await contract.view('get_user_limit_orders', { account_id: root });
    t.is(userOrders.length, 1);

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    await root.call(contract, 'remove_limit_order', {
        limit_order_id: limitOrderId,
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });

    const userOrdersAfter: any[] = await contract.view('get_user_limit_orders', { account_id: root });
    t.is(userOrdersAfter.length, 0);
});

test('limit orders: execute', async (t) => {
    const { root, tta, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add liquidity
    await mintLp(root, contract, near(300));
    await mintLpFtTransfer(root, contract, tta, '100000000000'); // $1000

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(30));

    const position: { collateral: string; size: string; } = await contract.view('get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Execute this one. Should close the position.
    await addDecreaseLimitOrder(root, contract, dollars(4), dollars(400), NEAR_ID, NEAR_ID, true, dollars(150), '0');

    // Should remain
    await addIncreaseLimitOrder(root, contract, dollars(6), dollars(200), NEAR_ID, true, near(20));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(4));

    const eligibleLimitOrders: string[] = await contract.view('get_eligible_orders', { asset_id: NEAR_ID });
    const userOrders: any[] = await contract.view('get_user_limit_orders', { account_id: root });
    t.is(eligibleLimitOrders.length, 1);
    t.is(userOrders.length, 2);

    await root.call(contract, 'execute_limit_order', {
        asset_id: NEAR_ID,
        limit_order_id: eligibleLimitOrders[0]
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });

    const eligibleLimitOrdersAfter: any[] = await contract.view('get_eligible_orders', { asset_id: NEAR_ID });
    const userOrdersAfter: any[] = await contract.view('get_user_limit_orders', { account_id: root });
    t.is(eligibleLimitOrdersAfter.length, 0);
    t.is(userOrdersAfter.length, 1);
});