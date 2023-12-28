#[cfg(test)]
mod tests {
    use ethers::types::U256;
    use mev_overwatch::mempool::extract_maybe_token_transfer;

    #[test]
    fn test_extract_maybe_token_transfer() {
        let tx_data = "0x573ade81000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000200000000000000000000000049b47081344e6208102ca69c6b3ae54e650885b6";

        // 573ade81 selector
        // 000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
        // ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        // 0000000000000000000000000000000000000000000000000000000000000002
        // 00000000000000000000000049b47081344e6208102ca69c6b3ae54e650885b6

        let result = extract_maybe_token_transfer(&tx_data);
        assert!(result.is_ok(), "Failed to extract token transfer");

        let (addresses, values) = result.unwrap();
        assert!(!addresses.is_empty(), "No addresses extracted");
        assert!(
            addresses.len() == 2,
            "Correct number of addresses extracted"
        );

        assert_eq!(
            addresses[0],
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                .parse()
                .unwrap(),
            "Address mismatch"
        );

        assert!(!values.is_empty(), "No values extracted");
        println!("{:?}", values[0]);
        assert!(values[0] == U256::max_value());
    }
}
