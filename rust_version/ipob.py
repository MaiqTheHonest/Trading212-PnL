import requests
import datetime
import json

ticker = "LACW"
start = int(datetime.datetime(2025, 9, 18).timestamp()) #"2025-08-18"
end = int(datetime.datetime(2025, 9, 22).timestamp()) #"2025-09-24"

url = "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?period1={start}&period2={end}&interval=1d".format(ticker=ticker, start=start, end=end)
header = {'User-Agent': 'Mozilla/5.0'}
response = requests.get(url, headers=header)

print(json.dumps(response.json(), indent=4))