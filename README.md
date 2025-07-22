Rust CLI tool for Trading 212 portoflio analysis.

Clarifies (un)realised P&L, money-weighted returns, dividends and calculates a few other metrics.

Trading 212 API allows **6x50** requests per minute, so mind the 1 minute wait if you have more than 300 orders.




<img width="434" height="302" alt="smallcroppedur" src="https://github.com/user-attachments/assets/7bb5d5d5-da70-4188-9dbb-6443fcd06442" /> <img width="350" height="302" alt="smalldiv3" src="https://github.com/user-attachments/assets/bbb8a4cd-f038-42ec-a0fd-496e340561de" />






Thank you [loony-bean](https://github.com/loony-bean) for `textplots`.




## Usage


Place a `.txt` file with your valid Trading212 API key into the same folder as `t212.exe` and launch executable (ideally via PowerShell or any other terminal that supports colour and UTF-8).


<br />
<br />

Alternatively, download the full rust_version, insert api key into the existing .txt file and `cargo run`.
