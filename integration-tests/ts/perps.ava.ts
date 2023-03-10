import {
    ONE_USD, TTA_DEC, NEAR_ID, near, dollars, updateIndexPrice, test, denom, storageDeposit,
    addAsset, NEAR_DEC, increasePosition, mintLp, mintLpFtTransfer, ftBalanceOf, decreasePosition
} from './utils.ava';
import { Worker, NEAR } from 'near-workspaces';

test('perps: short position', async (t) => {

    const { root, tta, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Add stable liquidity
    await mintLpFtTransfer(root, contract, tta, '100000000000'); // $1000

    // Create short position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), false, near(30));

    const position: { collateral: string; size: string; } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(4));

    const ttaBefore: string = await ftBalanceOf(tta, root);
    const nearAssetBefore: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetBefore: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: tta
    });

    // Decrease position with profit
    await decreasePosition(root, contract, positionId, dollars(10), dollars(20), false);

    const ttaAfter: string = await ftBalanceOf(tta, root);
    const positionAfterDecrease: { collateral: string; size: string; } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetAfter: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: tta
    });
    // $150 initial collateral - $10 delta
    t.is(positionAfterDecrease.collateral, dollars(140));
    t.is(positionAfterDecrease.size, dollars(380));

    t.is(
        nearAssetBefore.pool_amount, nearAssetAfter.pool_amount
    );
    // Protocol loses $4 on price changes: $20 size_delta * ($5 - $4) / $5 = $4 / 400 tokens
    t.is(
        BigInt(ttaAssetBefore.pool_amount) - BigInt(ttaAssetAfter.pool_amount), BigInt(4) * denom(TTA_DEC)
    );

    // User gets $10 delta + $4 profit
    t.is(
        BigInt(ttaAfter) - BigInt(ttaBefore), BigInt(14) * denom(TTA_DEC)
    );

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(6));

    // Decrease position with loss
    await decreasePosition(root, contract, positionId, dollars(10), dollars(20), false);

    const ttaAfter2: string = await ftBalanceOf(tta, root);
    const positionAfterDecrease2: { collateral: string; size: string; average_price: string } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter2: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetAfter2: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: tta
    });
    // $140 initial collateral - $10 delta - $4 loss
    t.is(positionAfterDecrease2.collateral, dollars(126));
    t.is(positionAfterDecrease2.size, dollars(360));
    t.is(nearAssetAfter.pool_amount, nearAssetAfter2.pool_amount);

    // Protocol gains $4 on price changes: $20 size_delta * ($5 - $4) / $5 = $4 / 400 tokens profit
    t.is(
        BigInt(ttaAssetAfter2.pool_amount) - BigInt(ttaAssetAfter.pool_amount), BigInt(4) * denom(TTA_DEC)
    );

    // User gets $10 delta 
    t.is(
        BigInt(ttaAfter2) - BigInt(ttaAfter), BigInt(10) * denom(TTA_DEC)
    );

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(6));

    // Close position with loss
    await decreasePosition(root, contract, positionId, dollars(0), dollars(360), false);

    const ttaAfter3: string = await ftBalanceOf(tta, root);
    const positionAfterDecrease3 = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter3: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    const ttaAssetAfter3: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: tta
    });

    // Position was closed
    t.is(positionAfterDecrease3, null);
    t.is(nearAssetAfter2.pool_amount, nearAssetAfter3.pool_amount);
    // Protocol gains $72 on price change: price $6, position price $5, size delta $360
    t.is(BigInt(ttaAssetAfter3.pool_amount) - BigInt(ttaAssetAfter2.pool_amount), BigInt(72) * denom(TTA_DEC));
    // User gets $126 collateral - $72 loss = $54
    t.is(BigInt(ttaAfter3) - BigInt(ttaAfter2), BigInt(54) * denom(TTA_DEC));
});

test('perps: long position', async (t) => {
    const { root, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(30));

    const position: { collateral: string; size: string; } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(6));

    const balance = await root.availableBalance();
    const nearAssetBefore: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });

    // Decrease position with profit
    await decreasePosition(root, contract, positionId, dollars(10), dollars(20), true);

    const balance2 = await root.availableBalance();
    const positionAfterDecrease: { collateral: string; size: string; } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    // $150 initial collateral - $10 delta
    t.is(positionAfterDecrease.collateral, dollars(140));
    t.is(positionAfterDecrease.size, dollars(380));

    // Collateral delta $10 - $4 = $6 / 2.33NEAR
    t.is(
        BigInt(nearAssetBefore.pool_amount) - BigInt(nearAssetAfter.pool_amount),
        BigInt('2333333333333333333333333')
    );
    // User gets $10 delta + $4 profit, price $6 = 2.33NEAR
    // Result is less due to sandbox issues
    t.assert(
        BigInt('2333333333333333333333333') - BigInt(balance2.sub(balance).toString())
        <= BigInt('1400000000000000000000') // 0.0014 NEAR
    );

    await updateIndexPrice(root, contract, NEAR_ID, dollars(4));

    // Decrease position with loss
    await decreasePosition(root, contract, positionId, dollars(10), dollars(20), true);

    const balance3 = await root.availableBalance();
    const positionAfterDecrease2: { collateral: string; size: string; average_price: string } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter2: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });
    // $140 initial collateral - $10 delta - $4 loss
    t.is(positionAfterDecrease2.collateral, dollars(126));
    t.is(positionAfterDecrease2.size, dollars(360));

    // Pool is less on $10 of collateral delta / 2.5NEAR 
    t.is(
        BigInt(nearAssetAfter.pool_amount) - BigInt(nearAssetAfter2.pool_amount),
        BigInt('2500000000000000000000000')
    );

    // User gets $10 delta 2.5 NEAR
    // Result is less due to sandbox issues
    t.assert(
        BigInt('2500000000000000000000000') - BigInt(balance3.sub(balance2).toString())
        <= BigInt('2600000000000000000000') // 0.0026 NEAR
    );

    await updateIndexPrice(root, contract, NEAR_ID, dollars(4));

    // Close position with loss
    await decreasePosition(root, contract, positionId, dollars(0), dollars(360), true);

    const balance4 = await root.availableBalance();
    const positionAfterDecrease3 = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const nearAssetAfter3: { pool_amount: string; } = await root.call(contract, 'get_asset_info', {
        asset: NEAR_ID
    });

    // Position was closed
    t.is(positionAfterDecrease3, null);
    // Decrease pool on collateral amount $126 - users loss $72 = $54 / 13.5NEAR
    t.is(
        BigInt(nearAssetAfter2.pool_amount) - BigInt(nearAssetAfter3.pool_amount),
        BigInt('13500000000000000000000000')
    );
    // User gets $126 collateral - $72 loss = $54 / 
    // Result is less due to sandbox issues
    t.assert(
        BigInt('13500000000000000000000000') - BigInt(balance4.sub(balance3).toString())
        <= BigInt('2600000000000000000000') // 0.0026 NEAR
    );
});

test('perps: custom output token', async (t) => {
    const { root, tta, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));
    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());

    // Add NEAR liquidity
    await mintLp(root, contract, near(300));

    // Add stable liquidity
    await mintLpFtTransfer(root, contract, tta, '100000000000'); // $1000

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));
    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());

    // Create long position
    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(30));

    const position: { collateral: string; size: string; } = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    t.is(position.collateral, dollars(150));
    t.is(position.size, dollars(400));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(6));
    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());

    const ttaBalanceBefore: string = await ftBalanceOf(tta, root);

    // Close position with profit
    await root.call(contract, 'decrease_position', {
        params: {
            position_id: positionId,
            collateral_delta: dollars(0),
            size_delta: dollars(400),
            is_long: true,
            output_token_id: tta.accountId
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });

    const positionAfterDecrease = await root.call(contract, 'get_position', {
        position_id: positionId
    });
    const ttaBalanceAfter: string = await ftBalanceOf(tta, root);

    // Position was closed
    t.is(positionAfterDecrease, null);
    // User gets $150 collateral + $80 profit in TTA stable asset
    // Small loss because of convertion 
    t.is(
        BigInt(ttaBalanceAfter) - BigInt(ttaBalanceBefore),
        BigInt('22999999999')
    );
});

test('failing scenarious: short position', async (t) => {
    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    const root = worker.rootAccount;
    console.log("deploying contracts...");
    const [tta, contract] = await Promise.all([
        root.devDeploy('./res/fungible_token.wasm', {
            method: 'new_default_meta',
            args: {
                owner_id: root,
                total_supply: NEAR.parse('1,000,000,000 N'),
            },
        }),
        root.devDeploy('./res/tonic_perps.wasm', {
            initialBalance: NEAR.parse('30 N').toJSON()
        }),
    ]);

    await contract.call(contract, 'new', { owner_id: root });

    const result = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result.receiptFailureMessagesContain('Contract is temporary paused'));

    // Enable contract 
    await root.call(contract, 'set_state', {
        state: 'Running'
    });

    const result2 = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result2.receiptFailureMessagesContain('Asset not found'));

    // add TTA and NEAR assets
    await addAsset(root, contract, tta.accountId, TTA_DEC, true, 50);
    await addAsset(root, contract, NEAR_ID, NEAR_DEC, false, 50);

    const result3 = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result3.receiptFailureMessagesContain('Contract is not correctly initialized.'));

    await root.call(contract, 'set_default_stablecoin', {
        asset_id: tta
    });

    const result4 = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result4.receiptFailureMessagesContain('Price should be greater than 0'));

    // set up root as admin and pay storage deposits
    await root.call(contract, 'add_price_oracle', {
        account_id: root,
    });

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    const result5 = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result5.receiptFailureMessagesContain('Not enough liquidity to perform swap'));

    await storageDeposit(root, tta, contract);

    const amount = BigInt(2000) * denom(TTA_DEC);
    await mintLpFtTransfer(root, contract, tta, amount.toString());

    const result6 = await root.callRaw(contract, 'increase_position', {
        params: {
            underlying_id: NEAR_ID,
            size_delta: dollars(400),
            is_long: false,
        },
    }, {
        attachedDeposit: near(30),
        gas: '300 TGas',
    });
    t.assert(result6.receiptFailureMessagesContain('Can not short asset'));

    await root.call(contract, 'set_shortable', {
        asset_id: NEAR_ID,
        shortable: true,
    });

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    let positionId = await increasePosition(root, contract, NEAR_ID, dollars(400), false, near(30));

    const result7 = await root.callRaw(contract, 'decrease_position', {
        params: {
            position_id: positionId,
            collateral_delta: dollars(10),
            size_delta: dollars(500),
            is_long: false,
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
    t.assert(result7.receiptFailureMessagesContain('Can not decrease position by more than size'));

    const result8 = await contract.callRaw(contract, 'decrease_position', {
        params: {
            position_id: positionId,
            collateral_delta: dollars(10),
            size_delta: dollars(500),
            is_long: false,
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
    t.assert(result8.receiptFailureMessagesContain('Can not decrease other account\'s position'));

    const result9 = await root.callRaw(contract, 'decrease_position', {
        params: {
            position_id: positionId,
            collateral_delta: dollars(1000),
            size_delta: dollars(100),
            is_long: false,
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
    t.assert(result9.receiptFailureMessagesContain('Can not take more than collateral out of position'));
});