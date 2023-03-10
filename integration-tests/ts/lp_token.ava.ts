import {
    ONE_USD, TTA_DEC, NEAR_ID, near, dollars, updateIndexPrice, test, denom, storageDeposit,
    addAsset, NEAR_DEC, increasePosition, mintLp, mintLpFtTransfer,
    TTB_DEC, LP_DEC, DOLLAR_DEC, burnLpToken, IncreasePositionFtTransfer, ftBalanceOf
} from './utils.ava';
import { Worker, NEAR } from 'near-workspaces';

test('mint and burn LP', async (t) => {

    const { root, tta, ttb, contract } = t.context.accounts;

    const priceA = ONE_USD;
    const priceB = dollars(2);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, ttb.accountId, priceB);

    const amtA = BigInt(1000);
    const amtB = BigInt(5000);

    // deposit A to get LP
    await mintLpFtTransfer(root, contract, tta, amtA.toString());

    const valueA = amtA * BigInt(priceA) / denom(TTA_DEC);
    const expectedLp0 = denom(LP_DEC) * valueA / denom(DOLLAR_DEC);

    t.is(BigInt(await ftBalanceOf(contract, root)), expectedLp0);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, ttb.accountId, priceB);

    // deposit B to get LP
    await mintLpFtTransfer(root, contract, ttb, amtB.toString());

    const valueB = amtB * BigInt(priceB) / denom(TTB_DEC);
    const expectedLp1 = expectedLp0 * valueB / valueA;

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, ttb.accountId, priceB);

    t.is(BigInt(await ftBalanceOf(contract, root)), expectedLp0 + expectedLp1);

    const amtBeforeA = BigInt(await ftBalanceOf(tta, root));
    const amtBeforeB = BigInt(await ftBalanceOf(ttb, root));

    const totalAumB = BigInt(await contract.view('get_total_aum', {}));
    const totalSupplyB = BigInt(await contract.view('ft_total_supply', {}));
    const amtOutB = expectedLp1 * totalAumB / totalSupplyB * denom(TTB_DEC) / BigInt(priceB);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, ttb.accountId, priceB);

    // burn LP to get B
    await burnLpToken(root, contract, expectedLp1.toString(), ttb.accountId);

    t.is(BigInt(await ftBalanceOf(contract, root)), expectedLp0);
    t.is(BigInt(await ftBalanceOf(ttb, root)), amtBeforeB + amtOutB);

    const totalAumA = BigInt(await contract.view('get_total_aum', {}));
    const totalSupplyA = BigInt(await contract.view('ft_total_supply', {}));
    const amtOutA = expectedLp0 * totalAumA / totalSupplyA * denom(TTA_DEC) / BigInt(priceA);

    await updateIndexPrice(root, contract, tta.accountId, priceA.toString());
    await updateIndexPrice(root, contract, ttb.accountId, priceB);

    // burn LP to get A
    await burnLpToken(root, contract, expectedLp0.toString(), tta.accountId);

    t.is(await ftBalanceOf(contract, root), '0');
    t.is(BigInt(await ftBalanceOf(tta, root)), amtBeforeA + amtOutA);
});

test('aum', async (t) => {
    const { root, tta, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    await root.call(contract, 'set_default_stablecoin', {
        asset_id: tta
    });
    await root.call(contract, 'set_shortable', {
        asset_id: NEAR_ID,
        shortable: true,
    });

    const amtA = BigInt(1000) * denom(TTA_DEC); // $1000

    // Add liquidity to stable pool
    await mintLpFtTransfer(root, contract, tta, amtA.toString());

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    const aum: string = await contract.view('get_total_aum');
    // Aum equals first deposit value
    t.is(aum, dollars(1000));

    // Add NEAR liquidity
    await mintLp(root, contract, near(1000));

    const aum2: string = await contract.view('get_total_aum');
    // $1000 (stable) + $5000 (NEAR)
    t.is(aum2, dollars(6000));

    // Create long position
    await increasePosition(root, contract, NEAR_ID, dollars(400), true, near(30));

    const aum3: string = await contract.view('get_total_aum');
    // Still $6000 as aum doesn't take into account position collateral
    t.is(aum3, dollars(6000));

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(6));

    const aum4: string = await contract.view('get_total_aum');
    // $1000 of stable + $5950 of NEAR ($250 guaranteed + $6180 pool (1030 NEAR) - $480 reserve (80 NEAR)) = $6950
    t.is(aum4, dollars(6950));

    let collateralAmount = (BigInt(100) * denom(TTA_DEC)).toString(); // $100

    // Create short position
    await IncreasePositionFtTransfer(root, contract.accountId, tta, collateralAmount, NEAR_ID, dollars(300), false);

    const aum5: string = await contract.view('get_total_aum');
    // $1000 of stable (short position collateral is not included into pool) + $5950 of NEAR
    t.is(aum5, dollars(6950));

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());
    await updateIndexPrice(root, contract, NEAR_ID, dollars(4));

    const aum6: string = await contract.view('get_total_aum');
    // $1000 of stable + $3950 of NEAR ($250 guaranteed + $4120 pool - $320 reserve - $100 short profits) = $3950
    t.is(aum6, dollars(4950));
});

test('failing scenarious: mint & burn', async (t) => {

    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    const root = worker.rootAccount;
    console.log("deploying contracts...");
    const contract = await root.devDeploy('./res/tonic_perps.wasm', {
        initialBalance: NEAR.parse('30 N').toJSON()
    });

    await contract.call(contract, 'new', { owner_id: root });

    const result = await root.callRaw(contract, 'mint_lp_near', {}, {
        attachedDeposit: near(300),
        gas: '300 TGas',
    });
    t.assert(result.receiptFailureMessagesContain('Contract is temporary paused'));

    // Enable contract 
    await root.call(contract, 'set_state', {
        state: 'Running'
    });

    const result2 = await root.callRaw(contract, 'mint_lp_near', {}, {
        attachedDeposit: near(300),
        gas: '300 TGas',
    });
    t.assert(result2.receiptFailureMessagesContain('Asset not found'));

    await addAsset(root, contract, NEAR_ID, NEAR_DEC, false, 50);

    const result3 = await root.callRaw(contract, 'mint_lp_near', {}, {
        attachedDeposit: near(300),
        gas: '300 TGas',
    });
    t.assert(result3.receiptFailureMessagesContain('Price should be greater than 0'));

    await root.call(contract, 'add_price_oracle', {
        account_id: root,
    });

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    let amount = (BigInt(100) * denom(TTA_DEC)).toString(); // $100

    const result5 = await root.callRaw(contract, 'burn_lp_token', {
        amount: amount,
        output_token_id: NEAR_ID,
    }, {
        attachedDeposit: '0',
        gas: '300 TGas',
    });
    t.assert(result5.receiptFailureMessagesContain('Price as unavailable due to lp supply absence'));

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    await mintLp(root, contract, near(300)); // $1500

    let amount2 = (BigInt(1400) * denom(LP_DEC)).toString(); // $1400

    await burnLpToken(root, contract, amount2, NEAR_ID);

    await updateIndexPrice(root, contract, NEAR_ID, dollars(5));

    const result6 = await root.callRaw(contract, 'burn_lp_token', {
        amount: amount2,
        output_token_id: NEAR_ID,
    }, {
        attachedDeposit: '0',
        gas: '300 TGas',
    });
    t.assert(result6.receiptFailureMessagesContain('The account doesn\'t have enough balance'));
});

test('ft transfer', async (t) => {
    const { root, tta, alice, contract } = t.context.accounts;

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());

    let metadata: { spec: string, decimals: number, name: string, symbol: string; } = await contract.view('ft_metadata');
    t.is(metadata.spec, 'ft-1.0.0');
    t.is(metadata.decimals, LP_DEC);
    t.is(metadata.name, 'Tonic Index LP Token');
    t.is(metadata.symbol, 'GIN');

    const amtA = BigInt(10000) * denom(TTA_DEC); // $10000

    // deposit A to get LP
    await mintLpFtTransfer(root, contract, tta, amtA.toString());

    let lp_amount = BigInt(10000) * denom(LP_DEC);
    t.is(BigInt(await contract.view('ft_total_supply')), lp_amount);
    t.is(BigInt(await ftBalanceOf(contract, root)), lp_amount);

    await root.call(contract, 'ft_transfer', {
        receiver_id: alice,
        amount: lp_amount.toString(),
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });

    t.is(BigInt(await ftBalanceOf(contract, alice)), lp_amount);
    t.is(await ftBalanceOf(contract, root), '0');

    await updateIndexPrice(root, contract, tta.accountId, ONE_USD.toString());

    await burnLpToken(alice, contract, lp_amount.toString(), tta.accountId);

    t.is(await ftBalanceOf(contract, alice), '0');
    t.is(await contract.view('ft_total_supply'), '0');
    t.is(BigInt(await ftBalanceOf(tta, alice)), amtA);
});

test('ft storage', async (t) => {

    const { root, contract } = t.context.accounts;

    const balanceBefore = await root.availableBalance();

    await storageDeposit(root, contract, root);

    const balanceAfter = await root.availableBalance();
    const storage: { total: string } = await contract.view('storage_balance_of', { account_id: root });

    t.assert(
        BigInt(balanceAfter.sub(balanceBefore).toString())
        <= BigInt('600000000000000000000') // 0.0006 NEAR
    );
    t.is(storage.total, '125000000000000000000000');
});