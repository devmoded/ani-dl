from base64 import b64decode

import json
import requests

TOKENS = "https://raw.githubusercontent.com/YaNesyTortiK/AnimeParsers/refs/heads/main/kdk_tokns/tokens.json"

def get_stable_encrypted_token() -> str:
    content = requests.get(TOKENS).text
    tokens = json.loads(content)
    return tokens.get("stable")[0].get("tokn")

def decrypt_token(tkn: str) -> str:
    p1 = tkn[: len(tkn) // 2][::-1]
    p2 = tkn[len(tkn) // 2 :][::-1]
    p1 = b64decode(p1.encode("utf-8"))
    p1 = p1.decode("utf-8")
    p2 = b64decode(p2.encode("utf-8"))
    p2 = p2.decode("utf-8")
    return p2 + p1

stable_token = decrypt_token(get_stable_encrypted_token())
print(stable_token)
