Rust program to extract the full order history, form a portfolio history from it and calculate realized / unrealized PnL using the buggy Trading212 API.

T212 allows **6x50** requests per minute => time delay for portfolio histories with more than 300 orders.

`t212 mod => main <= yahoo mod`
