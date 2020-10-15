import pandas as pd

df = pd.read_csv('data/yellow-tri/yellow_tripdata_2019-04.csv')

# Comment out the collumns you want to keep
df.drop('VendorID', axis=1, inplace=True)
#df.drop('tpep_pickup_datetime', axis=1, inplace=True)
df.drop('tpep_dropoff_datetime', axis=1, inplace=True)
df.drop('passenger_count', axis=1, inplace=True)
#df.drop('trip_distance', axis=1, inplace=True)
df.drop('RatecodeID', axis=1, inplace=True)
df.drop('store_and_fwd_flag', axis=1, inplace=True)
df.drop('PULocationID', axis=1, inplace=True)
df.drop('DOLocationID', axis=1, inplace=True)
df.drop('payment_type', axis=1, inplace=True)
df.drop('fare_amount', axis=1, inplace=True)
df.drop('extra', axis=1, inplace=True)
df.drop('mta_tax', axis=1, inplace=True)
df.drop('tip_amount', axis=1, inplace=True)
df.drop('tolls_amount', axis=1, inplace=True)
df.drop('improvement_surcharge', axis=1, inplace=True)
df.drop('total_amount', axis=1, inplace=True)
df.drop('congestion_surcharge', axis=1, inplace=True)

df['tpep_pickup_datetime'] = pd.to_datetime(df['tpep_pickup_datetime'])
df.drop_duplicates(subset=['tpep_pickup_datetime'], keep='first', inplace=True)
df.sort_values(by='tpep_pickup_datetime', inplace=True)
df.rename(columns={'tpep_pickup_datetime': 'timestamp', 'trip_distance': 'value'}, inplace=True)
df.to_csv('yellow_tripdata_2019-04_parsed.csv', index = False)