import { Worker, NEAR, NearAccount } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';

export const TTA_DEC = 8;
export const TTB_DEC = 10;
export const LP_DEC = 18;
export const DOLLAR_DEC = 6;
export const NEAR_DEC = 24;
export const NEAR_ID = 'near';
export const ONE_NEAR = denom(NEAR_DEC);
export const ONE_USD = BigInt('1000000');

export function denom(dec: number): bigint {
    return BigInt(10) ** BigInt(dec);
}

export function near(amount: number): string {
    return (BigInt(amount) * ONE_NEAR).toString();
}

export function dollars(amount: number): string {
    return (BigInt(amount) * ONE_USD).toString();
}

export async function updateIndexPrice(root: NearAccount, contract: NearAccount, assetId: string, price: string) {
    await root.call(contract, 'update_index_price', {
        reqs: [
            {
                asset_id: assetId,
                price: price,
            },
        ],
    });
}

export async function addDecreaseLimitOrder(
    root: NearAccount,
    contract: NearAccount,
    price: string,
    sizeDelta: string,
    underlyingId: string,
    collateralId: string,
    isLong: boolean,
    collateralDelta: string,
    attachedDeposit: string): Promise<string> {
    return await root.call(contract, 'add_limit_order', {
        params: {
            price: price,
            size_delta: sizeDelta,
            underlying_id: underlyingId,
            collateral_id: collateralId,
            is_long: isLong,
            order_type: 'Decrease',
            collateral_delta: collateralDelta,
        },
    }, {
        attachedDeposit: attachedDeposit,
        gas: '300 TGas',
    });
}

export async function addIncreaseLimitOrder(
    root: NearAccount,
    contract: NearAccount,
    price: string,
    size_delta: string,
    underlyingId: string,
    isLong: boolean,
    attachedDeposit: string) {
    await root.call(contract, 'add_limit_order', {
        params: {
            price: price,
            size_delta: size_delta,
            underlying_id: underlyingId,
            is_long: isLong,
            order_type: 'Increase',
        },
    }, {
        attachedDeposit: attachedDeposit,
        gas: '300 TGas',
    });
}

export const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
}>;

export async function storageDeposit(root: NearAccount, tokenAccount: NearAccount, accountId: NearAccount) {
    await root.call(tokenAccount, 'storage_deposit', {
        account_id: accountId,
    }, {
        attachedDeposit: NEAR.parse('1 N').toJSON(),
    });
}

export async function addAsset(
    root: NearAccount,
    contract: NearAccount,
    assetId: string,
    decimals: number,
    isStable: boolean,
    weight: number) {
    await root.call(contract, 'add_asset', {
        asset_id: assetId,
        decimals: decimals,
        stable: isStable,
        weight: weight,
    });
}

export async function setFeeParameters(root: NearAccount, contract: NearAccount, commonBps: number) {
    await root.call(contract, 'set_fee_parameters', {
        fee_parameters: {
            tax_bps: commonBps,
            stable_tax_bps: commonBps,
            mint_burn_fee_bps: commonBps,
            swap_fee_bps: commonBps,
            stable_swap_fee_bps: commonBps,
            margin_fee_bps: commonBps,
        },
    });
}

export async function increasePosition(
    root: NearAccount,
    contract: NearAccount,
    underlyingId: string,
    sizeDelta: string,
    isLong: boolean,
    attachedDeposit: string): Promise<string> {
    return await root.call(contract, 'increase_position', {
        params: {
            underlying_id: underlyingId,
            size_delta: sizeDelta,
            is_long: isLong,
        },
    }, {
        attachedDeposit: attachedDeposit,
        gas: '300 TGas',
    });
}

export async function mintLp(root: NearAccount, contract: NearAccount, attachedDeposit: string) {
    await root.call(contract, 'mint_lp_near', {}, {
        attachedDeposit: attachedDeposit,
        gas: '300 TGas',
    });
}

export async function mintLpFtTransfer(root: NearAccount, contract: NearAccount, token: NearAccount, amount: string) {
    await root.call(token, 'ft_transfer_call', {
        receiver_id: contract,
        amount: amount,
        msg: JSON.stringify({ action: 'MintLp', params: { min_out: '0' } }),
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
}

export async function liquidatePosition(liquidator: NearAccount, contract: NearAccount, positionId: string) {
    await liquidator.call(contract, 'liquidate_position', {
        params: {
            position_id: positionId
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
}

export async function burnLpToken(signer: NearAccount, contract: NearAccount, amount: string, outputToken: string) {
    await signer.call(contract, 'burn_lp_token', {
        amount: amount,
        output_token_id: outputToken,
    }, {
        attachedDeposit: '0',
        gas: '300 TGas',
    });
}

export async function IncreasePositionFtTransfer(
    root: NearAccount,
    contract: string,
    token: NearAccount,
    amount: string,
    underlyingId: string,
    sizeDelta: string,
    isLong: boolean
) {
    await root.call(token, 'ft_transfer_call', {
        receiver_id: contract,
        amount: amount,
        msg: JSON.stringify({
            action: 'IncreasePosition', params: {
                underlying_id: underlyingId,
                size_delta: sizeDelta,
                is_long: isLong,
            }
        }),
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
}

export async function ftBalanceOf(token: NearAccount, accountId: NearAccount): Promise<string> {
    return await token.view('ft_balance_of', { account_id: accountId });
}

export async function decreasePosition(
    root: NearAccount,
    contract: NearAccount,
    positionId: string,
    collateralDelta: string,
    sizeDelta: string,
    isLong: boolean,) {
    await root.call(contract, 'decrease_position', {
        params: {
            position_id: positionId,
            collateral_delta: collateralDelta,
            size_delta: sizeDelta,
            is_long: isLong,
        },
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
}

export async function swap(
    root: NearAccount,
    contract: NearAccount,
    token: NearAccount,
    amount: string,
    outputToken: string
) {
    await root.call(token, 'ft_transfer_call', {
        receiver_id: contract,
        amount: amount,
        msg: JSON.stringify({ action: 'Swap', params: { output_token_id: outputToken, min_out: '0' } }),
    }, {
        attachedDeposit: '1',
        gas: '300 TGas',
    });
}

test.beforeEach(async (t) => {
    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    const root = worker.rootAccount;
    console.log("deploying contracts...");
    const [tta, ttb, contract] = await Promise.all([
        root.devDeploy('./res/fungible_token.wasm', {
            method: 'new_default_meta',
            args: {
                owner_id: root,
                total_supply: NEAR.parse('1,000,000,000 N'),
            },
        }),
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

    console.log("creating test accounts...");
    const alice = await root.createSubAccount('alice', {
        initialBalance: NEAR.parse('30 N').toJSON(),
    });
    const bob = await root.createSubAccount('bob', {
        initialBalance: NEAR.parse('30 N').toJSON(),
    });
    const charlie = await root.createSubAccount('charlie', {
        initialBalance: NEAR.parse('30 N').toJSON(),
    });

    // Enable contract 
    await root.call(contract, 'set_state', {
        state: 'Running'
    });

    // set up root as admin and pay storage deposits
    await root.call(contract, 'add_price_oracle', {
        account_id: root,
    });
    // have to set fees to 0 or swap will panic due to asset liquidity drop check
    await setFeeParameters(root, contract, 0);

    await storageDeposit(root, tta, contract);
    await storageDeposit(root, ttb, contract);
    await storageDeposit(root, tta, alice);

    // add TTA and NEAR assets
    await addAsset(root, contract, tta.accountId, TTA_DEC, true, 50);
    await addAsset(root, contract, ttb.accountId, TTB_DEC, false, 50);
    await addAsset(root, contract, NEAR_ID, NEAR_DEC, false, 50);

    await root.call(contract, 'set_default_stablecoin', {
        asset_id: tta
    });
    await root.call(contract, 'set_shortable', {
        asset_id: NEAR_ID,
        shortable: true,
    });

    // Save state for test runs, it is unique for each test
    t.context.worker = worker;
    t.context.accounts = { root, tta, ttb, contract, alice, bob, charlie };
});

test.afterEach(async (t) => {
    // Stop Sandbox server
    await t.context.worker.tearDown().catch((error) => {
        console.log('Failed to stop the Sandbox:', error);
    });
});