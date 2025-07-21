Rust CLI tool for Trading 212 portoflio analysis.

Clarifies (un)realised P&L, money-weighted returns, dividends and calculates a few other metrics.

Trading 212 API allows **6x50** requests per minute, so mind the 1 minute wait if you have more than 300 orders.



![t212](https://github.com/user-attachments/assets/ce817dbd-9268-4380-aa27-f51f6b5161db)


<img width="783" height="697" alt="image" src="https://github.com/user-attachments/assets/fa951695-e5ea-410f-aca3-43d903f9e070" />



Thank you [loony-bean](https://github.com/loony-bean) for `textplots`.




## Usage


Place a `.txt` file with your valid Trading212 API key into the same folder as `t212.exe` and launch executable (ideally via PowerShell or any other terminal that supports colour and UTF-8).


<br />
<br />

Alternatively, download the full rust_version, insert api key into the existing .txt file and `cargo run`.
