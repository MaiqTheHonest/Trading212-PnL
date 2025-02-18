Rust program to extract the full order history, backtrace and synthesize the portfolio history from it, and calculate unrealized PnL, APR, average daily return and Sharpe ratio using the buggy Trading212 API.

T212 allows **6x50** requests per minute => time delay for portfolio histories with more than 300 orders.

<br />

`main => {t212, yahoo, plotter, dividends, stats}`

<br />

![t212](https://github.com/user-attachments/assets/ce817dbd-9268-4380-aa27-f51f6b5161db)




Thank you to [loony-bean](https://github.com/loony-bean) for `textplots`  (you should add x_tick customisation).


