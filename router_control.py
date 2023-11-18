"""
5G Router Controller
"""

import hashlib
import requests

ROUTER_HOST = "XXXXXX"
PASSWORD = "XXXXXXX"

GET_URL = f"http://{ROUTER_HOST}/goform/goform_get_cmd_process"
SET_URL = f"http://{ROUTER_HOST}/goform/goform_set_cmd_process"

r = requests.get(
    GET_URL,
    params={"isTest": "false", "cmd": "LD"},
    headers={"Referer": f"http://{ROUTER_HOST}/", "Host": ROUTER_HOST},
)

# print(r.json())

code = r.json()["LD"]

sha_sum = hashlib.sha256(
    (hashlib.sha256(PASSWORD.encode("ascii")).hexdigest().upper() + code).encode(
        "ascii"
    )
)

password = sha_sum.hexdigest().upper()
# print(password)

r = requests.post(
    SET_URL,
    headers={
        "Referer": f"http://{ROUTER_HOST}/",
        "Host": ROUTER_HOST,
        "Origin": f"http://{ROUTER_HOST}",
    },
    data={
        "isTest": "false",
        "goformId": "LOGIN",
        "password": password,
    },
    cookies={"stok": ""},
)

# print(r.json())
# print(r.cookies)
# print(r.headers)

cookies = r.cookies

r = requests.get(
    GET_URL,
    params={
        "isTest": "false",
        "multi_data": 1,
        "cmd": "loginfo,network_provider,network_type,signalbar",
    },
    headers={"Referer": f"http://{ROUTER_HOST}/", "Host": ROUTER_HOST},
    cookies=cookies,
)

print(r.json())
# print(r.cookies)
# print(r.headers)
