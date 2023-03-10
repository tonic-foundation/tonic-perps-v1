// @generated automatically by Diesel CLI.

pub mod perp_event {
    pub mod sql_types {
        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "edit_position_direction", schema = "perp_event"))]
        pub struct EditPositionDirection;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "edit_position_state", schema = "perp_event"))]
        pub struct EditPositionState;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "mint_burn_direction", schema = "perp_event"))]
        pub struct MintBurnDirection;
    }

    diesel::table! {
        perp_event.create_referral_code_event (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            code -> Text,
            created_at -> Nullable<Timestamp>,
        }
    }

    diesel::table! {
        perp_event.edit_fees (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            fee_native -> Text,
            fee_usd -> Text,
            fee_type -> Text,
            new_accumulated_fees_native -> Text,
            new_accumulated_fees_usd -> Text,
            increase -> Bool,
            asset_id -> Text,
        }
    }

    diesel::table! {
        perp_event.edit_guaranteed_usd (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            amount_usd -> Text,
            new_guaranteed_usd -> Text,
            increase -> Bool,
            asset_id -> Text,
        }
    }

    diesel::table! {
        perp_event.edit_pool_balance (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            amount_native -> Text,
            new_pool_balance_native -> Text,
            increase -> Bool,
            asset_id -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::EditPositionDirection;
        use super::sql_types::EditPositionState;

        perp_event.edit_position_event (id) {
            id -> Int4,
            receipt_id -> Text,
            account_id -> Text,
            block_timestamp -> Timestamp,
            position_id -> Text,
            direction -> EditPositionDirection,
            state -> EditPositionState,
            collateral_token -> Text,
            underlying_token -> Text,
            collateral_delta_native -> Text,
            size_delta_usd -> Text,
            new_size_usd -> Text,
            is_long -> Bool,
            price_usd -> Text,
            total_fee_usd -> Text,
            margin_fee_usd -> Text,
            position_fee_usd -> Text,
            total_fee_native -> Text,
            margin_fee_native -> Text,
            position_fee_native -> Text,
            usd_out -> Text,
            realized_pnl_to_date_usd -> Text,
            referral_code -> Nullable<Text>,
            adjusted_delta_usd -> Text,
            limit_order_id -> Nullable<Text>,
            collateral_delta_usd -> Text,
            liquidator_id -> Nullable<Text>,
        }
    }

    diesel::table! {
        perp_event.edit_reserved_amount (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            amount_native -> Text,
            new_reserved_amount_native -> Text,
            increase -> Bool,
            asset_id -> Text,
        }
    }

    diesel::table! {
        perp_event.indexer_processed_block (block_height) {
            block_height -> Int4,
            processed_at -> Nullable<Timestamp>,
        }
    }

    diesel::table! {
        perp_event.liquidate_position_event (id) {
            id -> Int4,
            block_timestamp -> Timestamp,
            receipt_id -> Text,
            liquidator_id -> Text,
            owner_id -> Text,
            position_id -> Text,
            collateral_token -> Text,
            underlying_token -> Text,
            is_long -> Bool,
            size_usd -> Text,
            collateral_usd -> Text,
            reserve_amount_delta_native -> Text,
            liquidation_price_usd -> Text,
            liquidator_reward_usd -> Text,
            liquidator_reward_native -> Text,
            fees_usd -> Text,
            fees_native -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::MintBurnDirection;

        perp_event.lp_mint_burn_event (id) {
            id -> Int4,
            receipt_id -> Text,
            account_id -> Text,
            block_timestamp -> Timestamp,
            direction -> MintBurnDirection,
            amount_in -> Text,
            amount_out -> Text,
            fees -> Text,
            fees_usd -> Text,
            fees_bps -> Int4,
            lp_price_usd -> Text,
            token_in -> Text,
            token_out -> Text,
        }
    }

    diesel::table! {
        perp_event.lp_price_update_event (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            price -> Text,
        }
    }

    diesel::table! {
        perp_event.place_limit_order_event (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            limit_order_id -> Text,
            collateral_token -> Text,
            underlying_token -> Text,
            order_type -> Text,
            threshold_type -> Text,
            collateral_delta_usd -> Text,
            attached_collateral_native -> Text,
            size_delta_usd -> Text,
            price_usd -> Text,
            expiry -> Timestamp,
            is_long -> Bool,
        }
    }

    diesel::table! {
        perp_event.remove_limit_order_event (id) {
            id -> Int4,
            account_id -> Text,
            block_timestamp -> Timestamp,
            receipt_id -> Text,
            underlying_token -> Text,
            limit_order_id -> Text,
            reason -> Text,
            liquidator_id -> Nullable<Text>,
        }
    }

    diesel::table! {
        perp_event.swap_event (id) {
            id -> Int4,
            account_id -> Text,
            block_timestamp -> Timestamp,
            receipt_id -> Text,
            token_in -> Text,
            token_out -> Text,
            amount_in_native -> Text,
            amount_out_native -> Text,
            fee_bps -> Int4,
            fees_usd -> Text,
            fees_native -> Text,
            amount_in_usd -> Text,
            amount_out_usd -> Text,
            referral_code -> Nullable<Text>,
        }
    }

    diesel::table! {
        perp_event.token_deposit_withdraw (id) {
            id -> Int4,
            receipt_id -> Text,
            block_timestamp -> Timestamp,
            account_id -> Text,
            amount_native -> Text,
            deposit -> Bool,
            method -> Text,
            receiver_id -> Text,
            asset_id -> Text,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        create_referral_code_event,
        edit_fees,
        edit_guaranteed_usd,
        edit_pool_balance,
        edit_position_event,
        edit_reserved_amount,
        indexer_processed_block,
        liquidate_position_event,
        lp_mint_burn_event,
        lp_price_update_event,
        place_limit_order_event,
        remove_limit_order_event,
        swap_event,
        token_deposit_withdraw,
    );
}
