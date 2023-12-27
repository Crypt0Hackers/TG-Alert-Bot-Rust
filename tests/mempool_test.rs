#[cfg(test)]
mod tests {
    use mev_overwatch::mempool::extract_token_transfer;

    #[test]
    fn test_extract_token_transfer() {
        let tx_data = "0x573ade81000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000200000000000000000000000049b47081344e6208102ca69c6b3ae54e650885b6";

        // 573ade81 selector
        // 000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
        // ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        // 0000000000000000000000000000000000000000000000000000000000000002
        // 00000000000000000000000049b47081344e6208102ca69c6b3ae54e650885b6

        let (address, ..) = extract_token_transfer(tx_data).unwrap();

        println!("address: {}", address);

        // 115792089237316195423570985008687907853269984665640564039457584007913129639935
        // 115792089237316195423570985008687907853269984665640564039457584007913129639935

        assert_eq!(
            address,
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                .parse()
                .unwrap()
        );
    }
}
