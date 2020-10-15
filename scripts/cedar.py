import pandas as pd

"""
Station Cedar Point Meteorological Data from jday 92 to 121
Metadata
Year: Year
Julian Day: Julian Day
Time: 24 hour clock Central Standard Time
Air Temperature: Temperature measured in celsius 
Wind Direction: Wind direction in degrees from North
Wind Speed: Wind speed measured in knots
Relative Humidity: Amount of water vapor (vapor pressure) in the air
Barometric Pressure: Barometric pressure measured in inches of mercury
Solar Radiation: Solar radiation measured in kW/m2
Quantum Radiation (PAR): Quantum radiation measured in �E/m2/s
Precipitation: Precipitation total measured in inches
Aggregate quality flag
0 or blank= quality not evaluated
1 = bad
2 = questionable/suspect
3 = good
 -9 = null value
"""

df = pd.read_csv('data/cedar/Cedar_point_met.csv')

# Comment out the collumns you want to keep
df.drop('stationid', axis=1, inplace=True)
df.drop('precip1', axis=1, inplace=True)
df.drop('precip1flag', axis=1, inplace=True)
df.drop('airtemp1', axis=1, inplace=True)
df.drop('airtemp1Flag', axis=1, inplace=True)
df.drop('solarrad1', axis=1, inplace=True)
df.drop('solarrad1Flag', axis=1, inplace=True)
#df.drop('quantumrad1', axis=1, inplace=True)
df.drop('quantumrad1Flag', axis=1, inplace=True)
df.drop('winddir1', axis=1, inplace=True)
df.drop('winddir1Flag', axis=1, inplace=True)
df.drop('windspeed1', axis=1, inplace=True)
df.drop('windspeed1Flag', axis=1, inplace=True)
df.drop('bar_pressure1', axis=1, inplace=True)
df.drop('bar_pressure1Flag', axis=1, inplace=True)
df.drop('relhumid1_avg', axis=1, inplace=True)
df.drop('relhumid1Flag', axis=1, inplace=True)

df['date'] = pd.to_datetime(df.yeardata, format='%Y') + pd.to_timedelta(df.jday - 1, unit='d')
df['timedata'] = pd.to_datetime(df['date'].astype(str) + df['timedata'].astype(str).apply(lambda x: x.zfill(4)), format="%Y-%m-%d%H%M")
df.drop('date', axis=1, inplace=True)
df.drop('yeardata', axis=1, inplace=True)
df.drop('jday', axis=1, inplace=True)
df.rename(columns={'timedata': 'timestamp', 'quantumrad1': 'value'}, inplace=True)
df.to_csv('data/cedar/cedar_point_quantumrad1.csv', index = False)
print("ok")

