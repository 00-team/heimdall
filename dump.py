import json
from urllib import request

TOKEN = "site 1:VNOEEARmGnaGctjUPW2vUuOAi76whSbHrXR2EXMTH"

DATA = [
    (200, 9),
    (404, 1),
    (200, 50),
    (400, 5),
    (501, 8),
    (200, 20),
    (200, 10),
    (200, 44),
]

DUMP = {
    "max_time": 0,
    "min_time": 0,
    "status": {},
    "total": 0,
    "total_time": 0,
}

for (status, time) in DATA:
    DUMP['total'] += 1
    DUMP['total_time'] += time

    if DUMP['min_time'] > time:
        DUMP['min_time'] = time

    if DUMP['max_time'] < time or DUMP['min_time'] == 0:
        DUMP['max_time'] = time

    sk = str(status)
    if sk in DUMP['status']:
        DUMP['status'][sk]['total_time'] += time
        DUMP['status'][sk]['count'] += 1

        if DUMP['status'][sk]['max_time'] < time:
            DUMP['status'][sk]['max_time'] = time

        if DUMP['status'][sk]['min_time'] > time:
            DUMP['status'][sk]['min_time'] = time
    else:
        DUMP['status'][sk] = {
            "code": status,
            "count": 1,
            "max_time": time,
            "min_time": time,
            "total_time": time
        }

rq = request.Request(
    url="http://localhost:7000/api/sites/dump/",
    method="POST",
    headers={"authorization": TOKEN, "content-type": "application/json"},
    data=json.dumps(DUMP).encode(),
)
request.urlopen(rq)
