from google.cloud import bigquery
# you need pandas to run this
# import pandas
import os

client = bigquery.Client()

lim = int(1e5)
query = 'SELECT address FROM bigquery-public-data.crypto_ethereum.balances where eth_balance > 0'
dfiter = client.query(query).result(page_size=lim).to_dataframe_iterable()
for df in dfiter:
    df.to_csv('accounts.csv', mode='a', header=not os.path.exists('accounts.csv'))

