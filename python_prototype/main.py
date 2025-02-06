import requests
import time
from datetime import datetime

# api key at the top for now so it doesnt have to be passed as arg to call every time
api_key = open("api_key.txt").read()
headers = {"Authorization": api_key}



def call_api(page: str, cursor: str = ""):

    query = {
    "cursor": cursor,
    "ticker": "",
    "limit": "50"
    }

    response = requests.get(page, headers=headers, params=query)
    
    try:

        results = response.json()
        print(results)
        results = results["items"]
        
        #iso_timestamp = [stuff["fillPrice"] for stuff in results["items"]]
        iso_timestamp = results[-1]["dateCreated"]
        nPP = int(datetime.strptime(iso_timestamp, "%Y-%m-%dT%H:%M:%S.%fZ").timestamp()) * 1000    # for milisecond-based timestamp that t212 needs

        return results, nPP
    except:
        print("--- no more data to be fetched")
        return None, None



url = "https://live.trading212.com/api/v0/equity/history/orders"
current_cursor = ''

order_counts = 0
while True:

    raw_orders, nextPagePath = call_api(url, current_cursor)
    
    if raw_orders is not None:
        order_counts += len(raw_orders)
        current_cursor = str(nextPagePath)
        # print(raw_orders)
        print(current_cursor)
        
    
    else:
        print(f"--- found {order_counts} orders, including cancelled")
        break


               

    

    
